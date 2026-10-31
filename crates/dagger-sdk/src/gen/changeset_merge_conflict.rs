//! Generated bindings owned by the GraphQL `ChangesetMergeConflict` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Strategy to use when merging changesets with conflicting changes."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ChangesetMergeConflict {
    #[doc = "Attempt the merge and fail if git merge fails due to conflicts"]
    #[serde(rename = "FAIL")]
    Fail,
    #[doc = "Fail before attempting merge if file-level conflicts are detected"]
    #[serde(rename = "FAIL_EARLY")]
    FailEarly,
    #[doc = "Let git create conflict markers in files. For modify/delete conflicts, keeps the modified version. Fails on binary conflicts."]
    #[serde(rename = "LEAVE_CONFLICT_MARKERS")]
    LeaveConflictMarkers,
    #[doc = "The conflict is resolved by applying the version of the calling changeset"]
    #[serde(rename = "PREFER_OURS")]
    PreferOurs,
    #[doc = "The conflict is resolved by applying the version of the other changeset"]
    #[serde(rename = "PREFER_THEIRS")]
    PreferTheirs,
}
