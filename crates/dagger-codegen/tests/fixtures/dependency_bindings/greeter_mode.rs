//! Bindings owned by visible GraphQL type `GreeterMode`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture enum GreeterMode.")]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    dagger_sdk :: __private :: serde :: Deserialize,
    dagger_sdk :: __private :: serde :: Serialize,
)]
#[serde(crate = "dagger_sdk::__private::serde")]
pub enum GreeterMode {
    #[cfg_attr(not(doctest), doc = "Client fixture value LOUD.")]
    #[serde(rename = "LOUD")]
    Loud,
    #[cfg_attr(not(doctest), doc = "Client fixture value QUIET.")]
    #[serde(rename = "QUIET")]
    Quiet,
}
