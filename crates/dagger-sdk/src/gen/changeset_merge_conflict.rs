//! Generated bindings owned by the GraphQL `ChangesetMergeConflict` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
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
impl crate::__private::ModuleInputCodec for ChangesetMergeConflict {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for ChangesetMergeConflict {
    fn encode_output<'a>(
        self,
        _context: &'a crate::__private::ModuleContextBase,
    ) -> crate::__private::ModuleBoxFuture<
        'a,
        Result<crate::__private::ModuleJson, crate::__private::EncodeError>,
    >
    where
        Self: 'a,
    {
        crate::__private::encode_string_value(self)
    }
}
