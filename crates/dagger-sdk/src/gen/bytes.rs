//! Generated bindings owned by the GraphQL `Bytes` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
