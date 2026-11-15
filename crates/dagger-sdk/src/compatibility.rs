//! Runtime-target validation for every implicit Dagger connection.
//!
//! The SDK's bindings are compiled against one Dagger API view, the target's. Before a
//! session becomes public the validator asks the engine for its release and for the
//! view it serves the session, and accepts any release no older than the target within
//! the target's major version that serves the target's view. An older release, another
//! major version and another view are each a known mismatch. They are decided before
//! any provenance evidence, so a known mismatch is never weakened into the explicit
//! escape hatch for genuinely unprovable provenance.

use std::cmp::Ordering;
use std::sync::Arc;

use semver::Version;
use serde_json::Value;

use crate::connection::EngineConnection;
use crate::graphql::{RawRequest, RawResponse, ResponseData};
use crate::runtime_errors::{
    CompatibilityError, CompatibilityErrorKind, CompatibilityEvidenceGap, CompatibilityTarget,
    ObservedIdentity,
};
use crate::target::{ExactTarget, exact_target};

const COMPATIBILITY_QUERY: &str = "query RustSdkCompatibility { version __schemaVersion }";
// An engine that predates the view field rejects the combined query. Its release alone
// is still enough to refuse it as older than the target.
const VERSION_ONLY_QUERY: &str = "query RustSdkCompatibility { version }";
const MAX_ECHOED_VIEW_LEN: usize = 64;

/// Stateless runtime-target validator assembled by the concrete connector.
#[derive(Clone, Debug)]
pub(crate) struct CompatibilityValidator {
    target: CompatibilityTarget,
}

/// One engine statement, before it is compared with the target.
struct Statement {
    version: Result<Version, CompatibilityEvidenceGap>,
    view: Result<Option<String>, CompatibilityEvidenceGap>,
}

impl CompatibilityValidator {
    pub(crate) fn exact() -> Result<Self, crate::TargetError> {
        Ok(Self::from_target(exact_target()?))
    }

    fn from_target(target: &ExactTarget) -> Self {
        Self {
            target: CompatibilityTarget {
                version: target.engine_version().clone(),
                revision_prefix: hex::encode(&target.revision().bytes()[..4]),
                view: target.schema_version().to_owned(),
            },
        }
    }

    pub(crate) async fn validate<C>(
        &self,
        connection: &C,
        allow_unverified: bool,
    ) -> Result<(), CompatibilityError>
    where
        C: EngineConnection + ?Sized,
    {
        let result = match self.statement(connection).await {
            Ok(statement) => self.check(statement),
            Err(error) => Err(error),
        };
        match result {
            Err(error)
                if error.kind() == CompatibilityErrorKind::Unverified && allow_unverified =>
            {
                Ok(())
            }
            other => other,
        }
    }

    async fn statement<C>(&self, connection: &C) -> Result<Statement, CompatibilityError>
    where
        C: EngineConnection + ?Sized,
    {
        let response = self.execute(connection, COMPATIBILITY_QUERY).await?;
        if response.errors().is_empty() {
            return Ok(Statement::read(response.data(), true));
        }
        let response = self.execute(connection, VERSION_ONLY_QUERY).await?;
        if response.errors().is_empty() {
            return Ok(Statement::read(response.data(), false));
        }
        Err(self.unverified(
            CompatibilityEvidenceGap::GraphQl,
            ObservedIdentity::default(),
            None,
        ))
    }

    async fn execute<C>(
        &self,
        connection: &C,
        query: &str,
    ) -> Result<RawResponse, CompatibilityError>
    where
        C: EngineConnection + ?Sized,
    {
        connection
            .execute(RawRequest::new(query))
            .await
            .map_err(|error| {
                self.unverified(
                    CompatibilityEvidenceGap::Transport,
                    ObservedIdentity::default(),
                    Some(Arc::new(error)),
                )
            })
    }

    /// Decides one engine statement: known mismatches first, then evidence gaps.
    fn check(&self, statement: Statement) -> Result<(), CompatibilityError> {
        let mut observed = ObservedIdentity::default();
        if let Ok(Some(view)) = &statement.view {
            observed.view = echoable_view(view);
        }
        if let Ok(version) = &statement.version {
            observed.version = Some(version.clone());
            observed.revision_prefix = clean_revision_prefix(version).map(str::to_owned);
            // Build metadata names the engine build, not the release, so it takes no
            // part in the floor.
            if version.cmp_precedence(&self.target.version) == Ordering::Less {
                return Err(self.mismatch(CompatibilityErrorKind::EngineTooOld, observed));
            }
            if version.major != self.target.version.major {
                return Err(self.mismatch(CompatibilityErrorKind::VersionMismatch, observed));
            }
        }
        if let Ok(Some(view)) = &statement.view
            && *view != self.target.view
        {
            return Err(self.mismatch(CompatibilityErrorKind::ViewMismatch, observed));
        }

        let version = match statement.version {
            Ok(version) => version,
            Err(gap) => return Err(self.unverified(gap, observed, None)),
        };
        match statement.view {
            Ok(Some(_)) => {}
            Ok(None) => {
                return Err(self.unverified(CompatibilityEvidenceGap::MissingView, observed, None));
            }
            Err(gap) => return Err(self.unverified(gap, observed, None)),
        }
        match revision_gap(&version) {
            Some(gap) => Err(self.unverified(gap, observed, None)),
            None => Ok(()),
        }
    }

    fn mismatch(
        &self,
        kind: CompatibilityErrorKind,
        observed: ObservedIdentity,
    ) -> CompatibilityError {
        CompatibilityError::mismatch(kind, &self.target, observed)
    }

    fn unverified(
        &self,
        gap: CompatibilityEvidenceGap,
        observed: ObservedIdentity,
        source: Option<Arc<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> CompatibilityError {
        CompatibilityError::unverified(gap, &self.target, observed, source)
    }
}

impl Statement {
    fn read(data: &ResponseData, view_requested: bool) -> Self {
        let object = match data {
            ResponseData::Value(Value::Object(object)) => object,
            ResponseData::Absent | ResponseData::Null => {
                return Self {
                    version: Err(CompatibilityEvidenceGap::MissingVersion),
                    view: Ok(None),
                };
            }
            ResponseData::Value(_) => {
                return Self {
                    version: Err(CompatibilityEvidenceGap::ResponseShape),
                    view: Ok(None),
                };
            }
        };
        let version = match object.get("version") {
            Some(Value::String(version)) => {
                Version::parse(version.strip_prefix('v').unwrap_or(version))
                    .map_err(|_| CompatibilityEvidenceGap::MalformedVersion)
            }
            None | Some(Value::Null) => Err(CompatibilityEvidenceGap::MissingVersion),
            Some(_) => Err(CompatibilityEvidenceGap::ResponseShape),
        };
        let view = if view_requested {
            match object.get("__schemaVersion") {
                Some(Value::String(view)) if !view.is_empty() => Ok(Some(view.clone())),
                Some(Value::String(_)) | None | Some(Value::Null) => Ok(None),
                Some(_) => Err(CompatibilityEvidenceGap::ResponseShape),
            }
        } else {
            Ok(None)
        };
        Self { version, view }
    }
}

/// Returns why a release's provenance cannot be proven, if it cannot.
fn revision_gap(version: &Version) -> Option<CompatibilityEvidenceGap> {
    let metadata = version.build.as_str();
    if metadata.is_empty() {
        return Some(CompatibilityEvidenceGap::MissingRevision);
    }
    if metadata
        .split(['.', '-'])
        .any(|component| component.eq_ignore_ascii_case("dirty"))
    {
        return Some(CompatibilityEvidenceGap::DirtyRevision);
    }
    // The metadata is the commit the engine was built from. Which commit is correct is
    // established by construction: the content-addressed build makes a release engine
    // the product of its declared source commit, so the runtime validator requires
    // clean, well-formed provenance and no pairing assertion.
    if clean_revision_prefix(version).is_none() {
        return Some(CompatibilityEvidenceGap::UnknownRevisionFormat);
    }
    None
}

fn clean_revision_prefix(version: &Version) -> Option<&str> {
    let metadata = version.build.as_str();
    (metadata.len() == 8
        && metadata
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then_some(metadata)
}

/// Keeps a stated view for messages only when it is shaped like a release name.
fn echoable_view(view: &str) -> Option<String> {
    let shaped = view.len() <= MAX_ECHOED_VIEW_LEN
        && view.strip_prefix('v').is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
        });
    shaped.then(|| view.to_owned())
}

#[cfg(test)]
impl CompatibilityValidator {
    /// Checks one stated release and view pair as if the engine had returned them.
    pub(crate) fn validate_identity(
        &self,
        version: &str,
        view: Option<&str>,
    ) -> Result<(), CompatibilityError> {
        let mut data = serde_json::Map::new();
        data.insert("version".to_owned(), Value::String(version.to_owned()));
        if let Some(view) = view {
            data.insert("__schemaVersion".to_owned(), Value::String(view.to_owned()));
        }
        self.check(Statement::read(
            &ResponseData::Value(Value::Object(data)),
            true,
        ))
    }

    pub(crate) fn expected_view(&self) -> &str {
        &self.target.view
    }
}
