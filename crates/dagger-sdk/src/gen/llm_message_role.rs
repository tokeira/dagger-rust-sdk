//! Generated bindings owned by the GraphQL `LLMMessageRole` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
