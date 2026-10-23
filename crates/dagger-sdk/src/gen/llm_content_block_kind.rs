//! Generated bindings owned by the GraphQL `LLMContentBlockKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "The kind of content in a message block."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum LlmContentBlockKind {
    #[doc = "Inline audio."]
    #[serde(rename = "AUDIO")]
    Audio,
    #[doc = "An inline PDF document."]
    #[serde(rename = "DOCUMENT")]
    Document,
    #[doc = "An inline image."]
    #[serde(rename = "IMAGE")]
    Image,
    #[doc = "Plain text content."]
    #[serde(rename = "TEXT")]
    Text,
    #[doc = "Model thinking/reasoning content (e.g. Anthropic extended thinking)."]
    #[serde(rename = "THINKING")]
    Thinking,
    #[doc = "A tool/function call from the model."]
    #[serde(rename = "TOOL_CALL")]
    ToolCall,
    #[doc = "A tool/function result."]
    #[serde(rename = "TOOL_RESULT")]
    ToolResult,
}
