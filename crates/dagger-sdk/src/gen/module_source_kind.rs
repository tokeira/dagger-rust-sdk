//! Generated bindings owned by the GraphQL `ModuleSourceKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "The kind of module source."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ModuleSourceKind {
    #[doc = "GraphQL enum value `DIR_SOURCE`."]
    #[serde(rename = "DIR_SOURCE", alias = "DIR")]
    DirSource,
    #[doc = "GraphQL enum value `GIT_SOURCE`."]
    #[serde(rename = "GIT_SOURCE", alias = "GIT")]
    GitSource,
    #[doc = "GraphQL enum value `LOCAL_SOURCE`."]
    #[serde(rename = "LOCAL_SOURCE", alias = "LOCAL")]
    LocalSource,
}
