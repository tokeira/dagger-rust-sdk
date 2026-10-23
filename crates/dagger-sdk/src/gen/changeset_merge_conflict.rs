//! Generated bindings owned by the GraphQL `ChangesetMergeConflict` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
