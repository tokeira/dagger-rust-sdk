//! Generated bindings owned by the GraphQL `WorkspaceCommitPickReason` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
