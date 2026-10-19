//! Generated bindings owned by the GraphQL `LLMMessageOriginKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
