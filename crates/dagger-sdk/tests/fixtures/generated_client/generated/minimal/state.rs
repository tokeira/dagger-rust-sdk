//! Generated bindings owned by GraphQL coordinate `MinimalState`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:cb1ab037106ae8c061b41d518b70d6e25710e8ac697b14d582ed73eea7291882","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Client fixture enum MinimalState."]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    dagger_sdk::__private::serde::Deserialize,
    dagger_sdk::__private::serde::Serialize,
)]
#[serde(crate = "dagger_sdk::__private::serde")]
pub enum State {
    #[doc = "Client fixture value BUSY."]
    #[serde(rename = "BUSY")]
    Busy,
    #[doc = "Client fixture value READY."]
    #[serde(rename = "READY")]
    Ready,
}
