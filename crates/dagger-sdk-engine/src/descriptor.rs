//! Cross-field validation for immutable packaged-engine coordinates.
//!
//! Scalar construction makes mutable dependency shapes unrepresentable. This module
//! adds the cross-field checks which bind registry packages to the exact SDK release
//! while allowing an independently immutable Git source, which content sealed from a
//! pinned commit uses before the SDK is on a registry.

use serde::Serialize;

use crate::canonical::{CanonicalError, DigestDomain, canonical_digest};
use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::{
    CanonicalRepositoryUrl, EngineSourceDescriptor, ExactRustToolchain, ExactVersion,
    FormatVersion, FullRevision, PublishedSdkDependency, Sha256Digest,
};

/// The descriptor coordinates that identify the generator on every engine platform.
#[derive(Serialize)]
struct GeneratorCoordinates<'a> {
    format_version: &'a FormatVersion,
    repository: &'a CanonicalRepositoryUrl,
    dagger_revision: &'a FullRevision,
    engine_version: &'a ExactVersion,
    rust_sdk_version: &'a ExactVersion,
    rust_toolchain: &'a ExactRustToolchain,
    sdk_dependency: &'a PublishedSdkDependency,
    core_schema_digest: &'a Sha256Digest,
}

impl EngineSourceDescriptor {
    /// Verifies cross-field invariants not already guaranteed by the scalar types.
    pub fn validate(&self) -> Result<(), EngineDiagnostic> {
        if let PublishedSdkDependency::Registry { exact_version, .. } = &self.sdk_dependency
            && exact_version != &self.rust_sdk_version
        {
            return Err(EngineDiagnostic::new(
                EngineDiagnosticCode::SdkManifestInvalid,
                Some("descriptor.sdk_dependency.exact_version"),
                "registry dependency version differs from the packaged Rust SDK version",
            ));
        }
        if self.repository.as_str().ends_with(".git") {
            return Err(EngineDiagnostic::new(
                EngineDiagnosticCode::SdkManifestInvalid,
                Some("descriptor.repository"),
                "repository identity must omit the transport-only .git suffix",
            ));
        }
        Ok(())
    }

    /// Identifies the generator independently of the engine's platform.
    ///
    /// Packaged content is sealed from the SDK source that `sdk_dependency` pins, and
    /// its tools are built with `rust_toolchain`, so these coordinates name the
    /// generator. The packaged asset digest stays out: it also covers the tools'
    /// executables, which differ by platform, so it would make files generated on one
    /// platform stale on every other.
    pub fn generator_digest(&self) -> Result<Sha256Digest, CanonicalError> {
        // Destructured so that a new descriptor field needs a decision here.
        let Self {
            format_version,
            repository,
            dagger_revision,
            engine_version,
            rust_sdk_version,
            rust_toolchain,
            sdk_dependency,
            core_schema_digest,
            packaged_asset_manifest_digest: _,
        } = self;
        canonical_digest(
            DigestDomain::Generator,
            &GeneratorCoordinates {
                format_version,
                repository,
                dagger_revision,
                engine_version,
                rust_sdk_version,
                rust_toolchain,
                sdk_dependency,
                core_schema_digest,
            },
        )
    }
}
