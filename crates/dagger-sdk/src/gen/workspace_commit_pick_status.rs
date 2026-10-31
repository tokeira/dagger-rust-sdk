//! Generated bindings owned by the GraphQL `WorkspaceCommitPickStatus` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
