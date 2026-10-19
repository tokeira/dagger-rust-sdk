//! Generated bindings owned by the GraphQL `ModuleSourceKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
