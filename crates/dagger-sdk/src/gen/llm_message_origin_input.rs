//! Generated bindings owned by the GraphQL `LLMMessageOriginInput` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "EXPERIMENTAL: Agent APIs are likely to change.\n\nThe provenance of a message delivered through an agent mailbox."]
#[derive(Clone, Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
#[non_exhaustive]
pub struct LlmMessageOriginInput {
    #[doc = "The display name of the sending or observed agent.\n\n**Experimental:** Agent APIs are likely to change."]
    #[serde(rename = "agentName", default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[doc = "Who put this message on the record.\n\n**Experimental:** Agent APIs are likely to change."]
    #[serde(rename = "kind")]
    pub kind: super::LlmMessageOriginKind,
    #[doc = "The message's short ref within the receiving agent's runtime, e.g. \"#3\".\n\n**Experimental:** Agent APIs are likely to change."]
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<String>,
    #[doc = "The ref of the message this one answers, if any.\n\n**Experimental:** Agent APIs are likely to change."]
    #[serde(rename = "replyTo", default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}
impl LlmMessageOriginInput {
    #[doc = "Creates `LlmMessageOriginInput` with every required GraphQL input field."]
    #[must_use]
    pub fn new(kind: super::LlmMessageOriginKind) -> Self {
        Self {
            agent_name: None,
            kind,
            r#ref: None,
            reply_to: None,
        }
    }
    #[doc = "Sets GraphQL input field `agentName`; the field is omitted until this method is called."]
    #[must_use]
    pub fn with_agent_name(mut self, value: String) -> Self {
        self.agent_name = Some(value);
        self
    }
    #[doc = "Sets GraphQL input field `ref`; the field is omitted until this method is called."]
    #[must_use]
    pub fn with_ref(mut self, value: String) -> Self {
        self.r#ref = Some(value);
        self
    }
    #[doc = "Sets GraphQL input field `replyTo`; the field is omitted until this method is called."]
    #[must_use]
    pub fn with_reply_to(mut self, value: String) -> Self {
        self.reply_to = Some(value);
        self
    }
}
