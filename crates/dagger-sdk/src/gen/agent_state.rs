//! Generated bindings owned by the GraphQL `AgentState` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
#[doc = "EXPERIMENTAL: Agent APIs are likely to change.\n\nComputed lifecycle state of an agent."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum AgentState {
    #[doc = "The loop failed; snapshot holds the completed prefix. Resume retries."]
    #[serde(rename = "FAILED")]
    Failed,
    #[doc = "Mailbox empty, turn complete; blocked in receive."]
    #[serde(rename = "IDLE")]
    Idle,
    #[doc = "Mailbox accepting but not draining, until resume."]
    #[serde(rename = "PAUSED")]
    Paused,
    #[doc = "A model request or tool evaluation is in flight."]
    #[serde(rename = "RUNNING")]
    Running,
    #[doc = "Runtime released; snapshot remains readable."]
    #[serde(rename = "STOPPED")]
    Stopped,
    #[doc = "Blocked on input from the user."]
    #[serde(rename = "WAITING_INPUT")]
    WaitingInput,
}
impl crate::__private::ModuleInputCodec for AgentState {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for AgentState {
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
