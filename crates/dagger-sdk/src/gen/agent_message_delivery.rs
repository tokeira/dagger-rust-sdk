//! Generated bindings owned by the GraphQL `AgentMessageDelivery` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "EXPERIMENTAL: Agent APIs are likely to change.\n\nHow a message landed in an agent's evaluation."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum AgentMessageDelivery {
    #[doc = "The message is queued: the agent is paused or failed, and a resume will drain it."]
    #[serde(rename = "QUEUED")]
    Queued,
    #[doc = "The message opened a new turn: the agent was idle or newly started."]
    #[serde(rename = "STARTED")]
    Started,
    #[doc = "The message was absorbed into the in-flight turn at a step boundary, steering it."]
    #[serde(rename = "STEERED")]
    Steered,
}
impl crate::__private::ModuleInputCodec for AgentMessageDelivery {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for AgentMessageDelivery {
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
