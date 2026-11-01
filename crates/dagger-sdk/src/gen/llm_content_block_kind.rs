//! Generated bindings owned by the GraphQL `LLMContentBlockKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
impl crate::__private::ModuleInputCodec for LlmContentBlockKind {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for LlmContentBlockKind {
    fn encode_output<'a>(
        self,
        _context: &'a crate::__private::ModuleContextBase,
    ) -> crate::__private::ModuleBoxFuture<
        'a,
        Result<crate::__private::ModuleJson, crate::__private::EncodeError>,
    >
    where
        Self: 'a,
    {
        crate::__private::encode_string_value(self)
    }
}
