//! Generated bindings owned by the GraphQL `CacheSharingMode` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Sharing mode of the cache volume."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum CacheSharingMode {
    #[doc = "Shares the cache volume amongst many build pipelines, but will serialize the writes"]
    #[serde(rename = "LOCKED")]
    Locked,
    #[doc = "Keeps a cache volume for a single build pipeline"]
    #[serde(rename = "PRIVATE")]
    Private,
    #[doc = "Shares the cache volume amongst many build pipelines"]
    #[serde(rename = "SHARED")]
    Shared,
}
