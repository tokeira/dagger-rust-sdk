//! Generated bindings owned by the GraphQL `ArtifactDimensionKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Generated enum for GraphQL `ArtifactDimensionKind`."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ArtifactDimensionKind {
    #[doc = "GraphQL enum value `COLLECTION`."]
    #[serde(rename = "COLLECTION")]
    Collection,
    #[doc = "GraphQL enum value `MODULE`."]
    #[serde(rename = "MODULE")]
    Module,
    #[doc = "GraphQL enum value `TYPE`."]
    #[serde(rename = "TYPE")]
    Type,
}
