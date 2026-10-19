//! Generated bindings owned by the GraphQL `WorkspaceCommitPickStatus` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Whether a source commit can be pulled."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum WorkspaceCommitPickStatus {
    #[doc = "The commit cannot be applied; see reason and conflictPaths."]
    #[serde(rename = "CONFLICT")]
    Conflict,
    #[doc = "The commit can be applied."]
    #[serde(rename = "PICKABLE")]
    Pickable,
    #[doc = "The commit is already present by hash or cherry-pick origin."]
    #[serde(rename = "PICKED")]
    Picked,
    #[doc = "The patch is already present, or applying it would be empty."]
    #[serde(rename = "REDUNDANT")]
    Redundant,
}
