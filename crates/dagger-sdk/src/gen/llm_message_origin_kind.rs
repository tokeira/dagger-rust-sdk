//! Generated bindings owned by the GraphQL `LLMMessageOriginKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
