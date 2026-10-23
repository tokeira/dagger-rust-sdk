//! Generated bindings owned by the GraphQL `ImageLayerCompression` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Compression algorithm to use for image layers."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ImageLayerCompression {
    #[doc = "GraphQL enum value `EStarGZ`."]
    #[serde(rename = "EStarGZ", alias = "ESTARGZ")]
    EStarGz,
    #[doc = "GraphQL enum value `Gzip`."]
    #[serde(rename = "Gzip", alias = "GZIP")]
    Gzip,
    #[doc = "GraphQL enum value `Uncompressed`."]
    #[serde(rename = "Uncompressed", alias = "UNCOMPRESSED")]
    Uncompressed,
    #[doc = "GraphQL enum value `Zstd`."]
    #[serde(rename = "Zstd", alias = "ZSTD")]
    Zstd,
}
