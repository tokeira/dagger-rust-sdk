//! Generated bindings owned by the GraphQL `DiffStatKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "The type of change for a diff stat entry."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum DiffStatKind {
    #[doc = "A file or directory was added."]
    #[serde(rename = "ADDED")]
    Added,
    #[doc = "A file was modified."]
    #[serde(rename = "MODIFIED")]
    Modified,
    #[doc = "A file or directory was removed."]
    #[serde(rename = "REMOVED")]
    Removed,
    #[doc = "A file was renamed."]
    #[serde(rename = "RENAMED")]
    Renamed,
}
