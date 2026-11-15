//! One checked visible-schema plan shared by every operation renderer.
//!
//! An engine-visible schema is verified against the reviewed core contract and then
//! consumed through the reviewed core definitions, so every engine that serves the
//! target's API view produces the same plan, bindings and schema identity.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use crate::ProjectionPlan;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticCoordinate, DiagnosticSet};
use crate::projection::catalog::SemanticDigest;
use crate::schema::canonical::{CanonicalSchema, SchemaCoordinate};
use crate::schema::{CoreCoordinateManifest, decode_and_validate, decode_and_validate_visible};
use crate::target::CodegenTarget;

use super::OperationKind;

const CHECKED_CORE_SCHEMA: &[u8] = include_bytes!("../../../../codegen/schema.json");

static CHECKED_CORE: OnceLock<Result<CheckedCore, DiagnosticSet>> = OnceLock::new();

/// The reviewed core schema and the coordinate manifests derived from it.
#[derive(Debug)]
struct CheckedCore {
    schema: CanonicalSchema,
    core: CoreCoordinateManifest,
    module: CoreCoordinateManifest,
}

// These are the exact module-introspection exclusions selected by core/moddeps.go and
// core/env.go at the checked target. ID types are scrubbed beside each raw type by the
// engine's schema builder.
const MODULE_HIDDEN_TYPES: &[&str] = &[
    "Host",
    "HostID",
    "Engine",
    "EngineID",
    "EngineCache",
    "EngineCacheID",
    "EngineCacheEntry",
    "EngineCacheEntryID",
    "EngineCacheEntrySet",
    "EngineCacheEntrySetID",
];
const MODULE_HIDDEN_FIELDS: &[&str] = &[
    "Query.currentWorkspace",
    "Query.engineVolume",
    "Query.sshfsVolume",
    "Query.setSessionTitle",
    "Address.volume",
];

/// Complete compatible schema an operation consumes and its single semantic projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisibleSchemaPlan {
    canonical: CanonicalSchema,
    core_coordinates: BTreeSet<SchemaCoordinate>,
    extension_coordinates: BTreeSet<SchemaCoordinate>,
    digest: SemanticDigest,
    projection: ProjectionPlan,
}

impl VisibleSchemaPlan {
    /// Returns the complete schema the operation consumes: the reviewed core
    /// definitions, as far as the visible schema has them, plus the visible schema's
    /// operation-scoped extensions.
    #[must_use]
    pub const fn canonical(&self) -> &CanonicalSchema {
        &self.canonical
    }

    /// Returns every coordinate owned by the checked core manifest.
    #[must_use]
    pub const fn core_coordinates(&self) -> &BTreeSet<SchemaCoordinate> {
        &self.core_coordinates
    }

    /// Returns every operation-scoped coordinate added around the core manifest.
    #[must_use]
    pub const fn extension_coordinates(&self) -> &BTreeSet<SchemaCoordinate> {
        &self.extension_coordinates
    }

    /// Returns the order-independent semantic identity of the consumed schema. Engines
    /// that serve the same reviewed core contract and the same extensions give the same
    /// identity, whatever documentation or unconsumed additions they serve beside it.
    #[must_use]
    pub const fn digest(&self) -> &SemanticDigest {
        &self.digest
    }

    /// Returns the single semantic projection consumed by all operation renderers.
    #[must_use]
    pub const fn projection(&self) -> &ProjectionPlan {
        &self.projection
    }

    /// Returns whether a named type is operation-scoped rather than checked core.
    #[must_use]
    pub fn is_extension_type(&self, name: &crate::schema::canonical::SchemaName) -> bool {
        self.extension_coordinates
            .contains(&SchemaCoordinate::named_type(name))
    }
}

/// Validates the complete visible schema and projects it exactly once.
pub fn project_visible_schema(
    target: &CodegenTarget,
    input: &[u8],
) -> Result<VisibleSchemaPlan, DiagnosticSet> {
    let checked = checked_core(target)?;
    project_visible_schema_with_manifest(target, input, checked, &checked.core, None)
}

pub(super) fn project_operation_visible_schema(
    target: &CodegenTarget,
    operation: OperationKind,
    input: &[u8],
    engine_version: Option<&str>,
) -> Result<VisibleSchemaPlan, DiagnosticSet> {
    let checked = checked_core(target)?;
    let manifest = if matches!(
        operation,
        OperationKind::GenerateModule | OperationKind::GenerateEntrypoint
    ) {
        &checked.module
    } else {
        &checked.core
    };
    project_visible_schema_with_manifest(target, input, checked, manifest, engine_version)
}

fn project_visible_schema_with_manifest(
    target: &CodegenTarget,
    input: &[u8],
    checked: &CheckedCore,
    manifest: &CoreCoordinateManifest,
    engine_version: Option<&str>,
) -> Result<VisibleSchemaPlan, DiagnosticSet> {
    let visible = decode_and_validate_visible(target, input, manifest, engine_version)?;
    let canonical = manifest.consumed_schema(&checked.schema, &visible);
    let all_coordinates = CoreCoordinateManifest::from_checked_schema(&canonical)?;
    let core_coordinates = manifest.coordinates().keys().cloned().collect();
    let extension_coordinates = all_coordinates
        .coordinates()
        .keys()
        .filter(|coordinate| !manifest.contains(coordinate))
        .cloned()
        .collect();
    let digest = SemanticDigest::for_value(&(
        canonical.query(),
        canonical.types(),
        canonical.directives(),
        canonical.inventory(),
    ))
    .map_err(|_| {
        DiagnosticSet::one(Diagnostic::new(
            DiagnosticCode::GeneratedProvenanceInvalid,
            Some(DiagnosticCoordinate::new("visible-schema")),
            "visible schema could not be fingerprinted",
        ))
    })?;
    let projected = crate::projection::project(&canonical)?;
    let projection = ProjectionPlan {
        target: target.clone(),
        schema: canonical.clone(),
        names: projected.names,
        named_types: projected.named_types,
        fields: projected.fields,
        directives: projected.directives,
        implementations: projected.implementations,
        catalog: projected.catalog,
    };
    Ok(VisibleSchemaPlan {
        canonical,
        core_coordinates,
        extension_coordinates,
        digest,
        projection,
    })
}

fn checked_core(target: &CodegenTarget) -> Result<&'static CheckedCore, DiagnosticSet> {
    CHECKED_CORE
        .get_or_init(|| {
            let schema = decode_and_validate(target, CHECKED_CORE_SCHEMA)?;
            let core = CoreCoordinateManifest::from_checked_schema(&schema)?;
            let module = CoreCoordinateManifest::from_checked_schema_with_scrub(
                &schema,
                MODULE_HIDDEN_TYPES,
                MODULE_HIDDEN_FIELDS,
            )?;
            Ok(CheckedCore {
                schema,
                core,
                module,
            })
        })
        .as_ref()
        .map_err(Clone::clone)
}
