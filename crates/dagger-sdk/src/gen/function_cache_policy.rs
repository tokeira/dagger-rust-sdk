//! Generated bindings owned by the GraphQL `FunctionCachePolicy` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "The behavior configured for function result caching."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum FunctionCachePolicy {
    #[doc = "GraphQL enum value `Default`."]
    #[serde(rename = "Default")]
    Default,
    #[doc = "GraphQL enum value `Never`."]
    #[serde(rename = "Never")]
    Never,
    #[doc = "GraphQL enum value `PerSession`."]
    #[serde(rename = "PerSession")]
    PerSession,
}
