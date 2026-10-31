//! Generated bindings owned by the GraphQL `PatchConflict` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "How to handle patch hunks that no longer apply to the target content."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum PatchConflict {
    #[doc = "Fail the operation if any part of the patch does not apply."]
    #[serde(rename = "FAIL")]
    Fail,
    #[doc = "Apply the hunks that fit and insert conflict markers where hunks no longer match, instead of failing."]
    #[serde(rename = "LEAVE_CONFLICT_MARKERS")]
    LeaveConflictMarkers,
}
