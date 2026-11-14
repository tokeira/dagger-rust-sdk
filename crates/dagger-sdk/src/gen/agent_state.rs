//! Generated bindings owned by the GraphQL `AgentState` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
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
