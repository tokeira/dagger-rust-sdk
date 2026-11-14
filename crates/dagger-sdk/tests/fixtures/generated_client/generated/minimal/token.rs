//! Generated bindings owned by GraphQL coordinate `MinimalToken`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:af5c520c5d560afb009cab87b651624ea148759019adbac1610ddcefcc3cee49","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
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
