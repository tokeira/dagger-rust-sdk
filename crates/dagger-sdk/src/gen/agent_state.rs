//! Generated bindings owned by the GraphQL `AgentState` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
