//! Generated bindings owned by GraphQL coordinate `MinimalState`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
