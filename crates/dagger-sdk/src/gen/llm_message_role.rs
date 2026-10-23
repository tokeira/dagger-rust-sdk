//! Generated bindings owned by the GraphQL `LLMMessageRole` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
