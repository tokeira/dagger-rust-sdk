//! Generated bindings owned by the GraphQL `LLMContentBlockKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
