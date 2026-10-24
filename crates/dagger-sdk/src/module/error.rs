//! Credential-safe structured application errors returned by authored module functions.

use std::collections::BTreeMap;
use std::error::Error;
use std::{fmt, io};

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    EngineConnectionError, QueryBuildError, QueryError, RequestEncodingError, RequestError,
    ResponseDecodingError,
};

const MAX_DETAIL_NAME_BYTES: usize = 128;
const MAX_DETAIL_COUNT: usize = 64;
const MAX_DETAILS_BYTES: usize = 8 * 1024;

/// One canonical JSON value attached to a module application error.
///
/// Values are intentionally owned. Generated adapters validate their target mapping
/// before publication; this type only provides a bounded, deterministic public
/// construction surface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ModuleErrorDetail(Value);

impl ModuleErrorDetail {
    /// Constructs a detail from an owned JSON value.
    #[must_use]
    pub const fn new(value: Value) -> Self {
        Self(value)
    }

    /// Returns the underlying JSON value for generated error projection.
    #[must_use]
    pub const fn as_json(&self) -> &Value {
        &self.0
    }

    /// Consumes the detail and returns its JSON value.
    #[must_use]
    pub fn into_json(self) -> Value {
        self.0
    }
}

impl From<Value> for ModuleErrorDetail {
    fn from(value: Value) -> Self {
        Self::new(value)
    }
}

impl From<String> for ModuleErrorDetail {
    fn from(value: String) -> Self {
        Self::new(Value::String(value))
    }
}

impl From<&str> for ModuleErrorDetail {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<bool> for ModuleErrorDetail {
    fn from(value: bool) -> Self {
        Self::new(Value::Bool(value))
    }
}

impl From<i64> for ModuleErrorDetail {
    fn from(value: i64) -> Self {
        Self::new(Value::Number(value.into()))
    }
}

impl From<u64> for ModuleErrorDetail {
    fn from(value: u64) -> Self {
        Self::new(Value::Number(value.into()))
    }
}

/// Failure to construct a bounded structured [`ModuleError`].
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleErrorBuildError {
    /// A detail name was empty or exceeded the documented bound.
    InvalidDetailName,
    /// The number of distinct details exceeded the documented bound.
    TooManyDetails,
    /// Canonical JSON detail content exceeded the documented byte bound.
    DetailsTooLarge,
}

impl fmt::Display for ModuleErrorBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDetailName => "the module error detail name is invalid",
            Self::TooManyDetails => "the module error has too many details",
            Self::DetailsTooLarge => "the module error details are too large",
        })
    }
}

impl Error for ModuleErrorBuildError {}

/// An intentional application failure returned by authored module code.
///
/// `Display` renders only the caller-selected message. Details remain available
/// through typed inspection and are projected in sorted key order by generated code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleError {
    message: String,
    details: BTreeMap<String, ModuleErrorDetail>,
}

impl ModuleError {
    /// Constructs an application error without structured details.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: BTreeMap::new(),
        }
    }

    /// Returns the intentional application-facing message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Iterates over details in canonical key order.
    pub fn details(&self) -> impl ExactSizeIterator<Item = (&str, &ModuleErrorDetail)> {
        self.details
            .iter()
            .map(|(name, detail)| (name.as_str(), detail))
    }

    /// Adds or replaces one bounded structured detail.
    ///
    /// Validation occurs before `self` is changed, so a failure leaves no partially
    /// constructed error value observable to the caller.
    pub fn with_detail(
        mut self,
        name: impl Into<String>,
        value: ModuleErrorDetail,
    ) -> Result<Self, ModuleErrorBuildError> {
        let name = name.into();
        if name.is_empty() || name.len() > MAX_DETAIL_NAME_BYTES {
            return Err(ModuleErrorBuildError::InvalidDetailName);
        }
        if !self.details.contains_key(&name) && self.details.len() == MAX_DETAIL_COUNT {
            return Err(ModuleErrorBuildError::TooManyDetails);
        }

        let previous = self.details.insert(name.clone(), value);
        let encoded = serde_json::to_vec(&self.details)
            .map_err(|_| ModuleErrorBuildError::DetailsTooLarge)?;
        if encoded.len() > MAX_DETAILS_BYTES {
            if let Some(previous) = previous {
                self.details.insert(name, previous);
            } else {
                self.details.remove(&name);
            }
            return Err(ModuleErrorBuildError::DetailsTooLarge);
        }
        Ok(self)
    }
}

impl fmt::Display for ModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ModuleError {}

impl From<String> for ModuleError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

impl From<&str> for ModuleError {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}

impl From<QueryError> for ModuleError {
    fn from(error: QueryError) -> Self {
        match &error {
            QueryError::GraphQl { response } => {
                if let [graphql_error] = response.errors() {
                    return Self::with_engine_extensions(
                        graphql_error.message(),
                        graphql_error.extensions(),
                    );
                }
            }
            // The engine's message ends with the failed command's trace origin, which
            // the CLI follows to show that command and its output, so it stays verbatim.
            QueryError::Exec { error: exec, .. } => {
                return Self::with_engine_extensions(exec.message(), Some(exec.extensions()));
            }
            // An argument that failed to resolve carries the engine's own error.
            QueryError::Build(_) => {
                if let Some(cause) = query_cause(&error) {
                    return Self::from(cause.clone());
                }
            }
            _ => {}
        }
        Self::new(sdk_causes(&error))
    }
}

impl ModuleError {
    /// An engine error's message with each of its extensions that fits the detail
    /// bounds. One too large to keep is left out, so the bounds always hold.
    fn with_engine_extensions(message: &str, extensions: Option<&Map<String, Value>>) -> Self {
        let mut error = Self::new(message);
        for (name, value) in extensions.into_iter().flatten() {
            if let Ok(updated) = error
                .clone()
                .with_detail(name.clone(), ModuleErrorDetail::new(value.clone()))
            {
                error = updated;
            }
        }
        error
    }
}

/// The first query error below `error` in its cause chain.
fn query_cause(error: &QueryError) -> Option<&QueryError> {
    let mut cause = Error::source(error);
    while let Some(current) = cause {
        if let Some(query) = current.downcast_ref::<QueryError>() {
            return Some(query);
        }
        cause = current.source();
    }
    None
}

/// The display of `error` and of each SDK error below it, then the I/O failure at the
/// root of the chain, if any. The chain stops at the first error from another crate, such
/// as a transport library's: only the SDK's own renderings are guaranteed free of
/// credentials and response bodies.
fn sdk_causes(error: &QueryError) -> String {
    let mut message = error.to_string();
    let mut cause = Error::source(error);
    while let Some(current) = cause {
        if !(current.is::<QueryError>()
            || current.is::<RequestError>()
            || current.is::<EngineConnectionError>()
            || current.is::<QueryBuildError>()
            || current.is::<RequestEncodingError>()
            || current.is::<ResponseDecodingError>())
        {
            break;
        }
        message.push_str(": ");
        message.push_str(&current.to_string());
        cause = current.source();
    }
    if let Some(failure) = cause.and_then(io_failure) {
        message.push_str(": ");
        message.push_str(&failure);
    }
    message
}

/// The kind of the deepest I/O error in `error`'s chain, with its operating system code if
/// it has one, such as "timed out (os error 110)". Both come from fixed tables, unlike the
/// errors' own renderings, which can carry a URL, a header or a response. An error of kind
/// `Other` without a code says nothing a reader can act on, so it gives `None`.
fn io_failure(error: &(dyn Error + 'static)) -> Option<String> {
    let mut deepest = None;
    let mut cause = Some(error);
    while let Some(current) = cause {
        cause = match current.downcast_ref::<io::Error>() {
            Some(failure) => {
                deepest = Some(failure);
                // An I/O error's `source` skips the error it wraps, which may be another
                // I/O error.
                failure
                    .get_ref()
                    .map(|inner| inner as &(dyn Error + 'static))
            }
            None => current.source(),
        };
    }
    let failure = deepest?;
    match (failure.kind(), failure.raw_os_error()) {
        (io::ErrorKind::Other, None) => None,
        (kind, None) => Some(kind.to_string()),
        (kind, Some(code)) => Some(format!("{kind} (os error {code})")),
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::{fmt, io};

    use serde_json::{Map, json};

    use crate::graphql::{GraphQlError, RawResponse, ResponseData};
    use crate::{
        EngineConnectionError, EngineConnectionErrorKind, ExecError, QueryBuildError,
        QueryBuildErrorKind, QueryError, RequestError,
    };

    use super::{ModuleError, ModuleErrorBuildError, ModuleErrorDetail};

    const ORIGIN: &str = " [traceparent:0123456789abcdef0123456789abcdef-0123456789abcdef]";

    fn exec_failure(stdout: &str) -> QueryError {
        let response = RawResponse::new(ResponseData::Null).with_errors(vec![
            GraphQlError::new(format!(
                "process \"false\" did not complete successfully: exit code: 1{ORIGIN}"
            ))
            .with_extensions(Map::from_iter([
                ("_type".to_owned(), json!("EXEC_ERROR")),
                ("cmd".to_owned(), json!(["false"])),
                ("exitCode".to_owned(), json!(1)),
                ("stdout".to_owned(), json!(stdout)),
                ("stderr".to_owned(), json!("failed")),
            ])),
        ]);
        QueryError::Exec {
            error: ExecError::from_response(&response).expect("the response is an exec failure"),
            response,
        }
    }

    #[test]
    fn details_are_sorted_and_display_is_only_the_message() {
        let error = ModuleError::new("not found")
            .with_detail("zeta", ModuleErrorDetail::from(2_i64))
            .expect("first detail is bounded")
            .with_detail("alpha", ModuleErrorDetail::from(true))
            .expect("second detail is bounded");

        assert_eq!(error.to_string(), "not found");
        assert_eq!(
            error.details().map(|(name, _)| name).collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
    }

    #[test]
    fn invalid_detail_name_is_rejected() {
        assert_eq!(
            ModuleError::new("invalid").with_detail("", ModuleErrorDetail::from(false)),
            Err(ModuleErrorBuildError::InvalidDetailName)
        );
    }

    #[test]
    fn one_graphql_error_retains_its_message_and_sorted_extensions() {
        let response = RawResponse::new(ResponseData::Null).with_errors(vec![
            GraphQlError::new("application failed").with_extensions(Map::from_iter([
                ("zeta".to_owned(), json!(2)),
                ("alpha".to_owned(), json!(true)),
            ])),
        ]);
        let error = ModuleError::from(QueryError::GraphQl { response });

        assert_eq!(error.message(), "application failed");
        assert_eq!(
            error.details().map(|(name, _)| name).collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
    }

    // The trace origin at the end of the engine's message is how the CLI finds the
    // failed command and shows its output.
    #[test]
    fn an_exec_failure_keeps_the_engine_message_and_its_origin() {
        let error = ModuleError::from(exec_failure("ran"));

        assert_eq!(
            error.message(),
            format!("process \"false\" did not complete successfully: exit code: 1{ORIGIN}")
        );
        assert_eq!(
            error.details().map(|(name, _)| name).collect::<Vec<_>>(),
            ["_type", "cmd", "exitCode", "stderr", "stdout"]
        );
    }

    #[test]
    fn an_exec_failure_keeps_every_detail_that_fits() {
        let error = ModuleError::from(exec_failure(&"x".repeat(16 * 1024)));

        assert!(error.message().ends_with(ORIGIN));
        assert_eq!(
            error.details().map(|(name, _)| name).collect::<Vec<_>>(),
            ["_type", "cmd", "exitCode", "stderr"]
        );
    }

    #[test]
    fn an_argument_that_failed_to_resolve_reports_the_engine_error() {
        let response = RawResponse::new(ResponseData::Null).with_errors(vec![GraphQlError::new(
            "acceptance: no such file or directory",
        )]);
        let error = ModuleError::from(QueryError::Build(QueryBuildError::with_source(
            QueryBuildErrorKind::LazyIdentifier,
            QueryError::GraphQl { response },
        )));

        assert_eq!(error.message(), "acceptance: no such file or directory");
    }

    #[test]
    fn a_request_failure_names_its_sdk_causes_and_not_the_transport_error() {
        let transport = io::Error::other("connection reset; Authorization: Bearer secret");
        let error = ModuleError::from(QueryError::Request(RequestError::Connection(
            EngineConnectionError::with_source(EngineConnectionErrorKind::Transport, transport),
        )));

        assert_eq!(
            error.message(),
            "the GraphQL query request failed: the engine connection failed: the engine transport failed"
        );
    }

    /// A transport library's error, whose rendering names the URL it failed to reach.
    #[derive(Debug)]
    struct LibraryError(io::Error);

    impl fmt::Display for LibraryError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("error sending request for url (http://user:secret@127.0.0.1/)")
        }
    }

    impl Error for LibraryError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    fn transport_failure(source: impl Error + Send + Sync + 'static) -> QueryError {
        QueryError::Request(RequestError::Connection(
            EngineConnectionError::with_source(EngineConnectionErrorKind::Transport, source),
        ))
    }

    #[test]
    fn a_request_failure_names_the_kind_of_the_io_failure_below_the_library() {
        let wrapped = io::Error::other(io::Error::from(io::ErrorKind::TimedOut));
        let error = ModuleError::from(transport_failure(LibraryError(wrapped)));

        assert_eq!(
            error.message(),
            "the GraphQL query request failed: the engine connection failed: the engine transport failed: timed out"
        );
    }

    #[test]
    fn a_request_failure_names_the_operating_system_code() {
        let failure = io::Error::from_raw_os_error(110);
        let kind = failure.kind();
        let error = ModuleError::from(transport_failure(LibraryError(failure)));

        assert_eq!(
            error.message(),
            format!(
                "the GraphQL query request failed: the engine connection failed: the engine transport failed: {kind} (os error 110)"
            )
        );
    }
}
