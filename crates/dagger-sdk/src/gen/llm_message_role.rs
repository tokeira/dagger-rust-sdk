//! Generated bindings owned by the GraphQL `LLMMessageRole` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "The role that generated a message."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum LlmMessageRole {
    #[doc = "A reply from the model."]
    #[serde(rename = "ASSISTANT")]
    Assistant,
    #[doc = "A system prompt."]
    #[serde(rename = "SYSTEM")]
    System,
    #[doc = "A user prompt or tool response."]
    #[serde(rename = "USER")]
    User,
}
