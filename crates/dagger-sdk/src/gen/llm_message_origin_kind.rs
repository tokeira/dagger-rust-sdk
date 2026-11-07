//! Generated bindings owned by the GraphQL `LLMMessageOriginKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
#[doc = "EXPERIMENTAL: Agent APIs are likely to change.\n\nWho put a message on the conversation record."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum LlmMessageOriginKind {
    #[doc = "Another agent: the message was sent from within that agent's turn."]
    #[serde(rename = "AGENT")]
    Agent,
    #[doc = "The engine, reporting a subscribed agent's lifecycle transition."]
    #[serde(rename = "EVENT")]
    Event,
    #[doc = "The user: a prompt submitted by a client rather than sent by an agent."]
    #[serde(rename = "USER")]
    User,
}
impl crate::__private::ModuleInputCodec for LlmMessageOriginKind {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for LlmMessageOriginKind {
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
