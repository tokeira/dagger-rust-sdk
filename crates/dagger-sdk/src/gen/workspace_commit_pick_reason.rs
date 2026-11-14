//! Generated bindings owned by the GraphQL `WorkspaceCommitPickReason` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "Why a source commit cannot be pulled."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum WorkspaceCommitPickReason {
    #[doc = "The patch conflicts with committed content."]
    #[serde(rename = "CONTENT")]
    Content,
    #[doc = "The commit touches uncommitted paths in the receiving workspace."]
    #[serde(rename = "DIRTY")]
    Dirty,
    #[doc = "No conflict."]
    #[serde(rename = "NONE")]
    None,
}
impl crate::__private::ModuleInputCodec for WorkspaceCommitPickReason {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for WorkspaceCommitPickReason {
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
