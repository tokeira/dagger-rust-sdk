//! Generated bindings owned by the GraphQL `WorkspaceCommitPickReason` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
