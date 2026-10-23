//! Generated bindings owned by the GraphQL `AgentState` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
