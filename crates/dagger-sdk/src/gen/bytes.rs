//! Generated bindings owned by the GraphQL `Bytes` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Arbitrary binary data, represented as a base64-encoded string."]
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
#[serde(transparent)]
pub struct Bytes(
    #[doc = r" Exact string retained for GraphQL request and response encoding."] pub String,
);
impl From<String> for Bytes {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Bytes {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
