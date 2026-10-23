//! Generated bindings owned by the GraphQL `ReturnType` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Expected return type of an execution"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ReturnType {
    #[doc = "Any execution (exit codes 0-127 and 192-255)"]
    #[serde(rename = "ANY")]
    Any,
    #[doc = "A failed execution (exit codes 1-127 and 192-255)"]
    #[serde(rename = "FAILURE")]
    Failure,
    #[doc = "A successful execution (exit code 0)"]
    #[serde(rename = "SUCCESS")]
    Success,
}
