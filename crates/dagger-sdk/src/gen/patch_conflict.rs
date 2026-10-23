//! Generated bindings owned by the GraphQL `PatchConflict` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
