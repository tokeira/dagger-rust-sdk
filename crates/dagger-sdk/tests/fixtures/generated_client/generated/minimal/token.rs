//! Generated bindings owned by GraphQL coordinate `MinimalToken`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:e0fa5b0e88fd7137b64e4af00205e21391e92aa630b8bd882b594fc4b32c6a6b","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
