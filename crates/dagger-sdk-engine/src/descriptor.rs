//! Cross-field validation for immutable packaged-engine coordinates.
//!
//! Scalar construction makes mutable dependency shapes unrepresentable. This module
//! adds the cross-field checks which bind registry packages to the exact SDK release
//! while allowing an independently immutable Git source, which content sealed from a
//! pinned commit uses before the SDK is on a registry.

use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::{EngineSourceDescriptor, PublishedSdkDependency};

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
}
