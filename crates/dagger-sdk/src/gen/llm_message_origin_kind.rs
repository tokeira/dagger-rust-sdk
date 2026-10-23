//! Generated bindings owned by the GraphQL `LLMMessageOriginKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
