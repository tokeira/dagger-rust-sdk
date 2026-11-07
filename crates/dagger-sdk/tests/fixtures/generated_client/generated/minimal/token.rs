//! Generated bindings owned by GraphQL coordinate `MinimalToken`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:506c57349967eeaceb1235a4465193229f99ff7b15c2a58a4e8fd04be3d5008f","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
