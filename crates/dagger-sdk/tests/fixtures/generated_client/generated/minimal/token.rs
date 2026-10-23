//! Generated bindings owned by GraphQL coordinate `MinimalToken`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Client fixture scalar MinimalToken."]
#[derive(
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    dagger_sdk::__private::serde::Deserialize,
    dagger_sdk::__private::serde::Serialize,
)]
#[serde(crate = "dagger_sdk::__private::serde", transparent)]
pub struct Token(
    /// Exact string retained for GraphQL request and response encoding.
    pub String,
);
impl From<String> for Token {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
