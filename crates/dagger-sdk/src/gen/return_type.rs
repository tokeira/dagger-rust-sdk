//! Generated bindings owned by the GraphQL `ReturnType` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
