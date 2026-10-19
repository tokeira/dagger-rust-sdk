//! Generated bindings owned by the GraphQL `LLMMessageOrigin` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "EXPERIMENTAL: Agent APIs are likely to change.\n\nThe recorded provenance of a message that arrived through an agent mailbox."]
#[derive(Clone)]
pub struct LlmMessageOrigin {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for LlmMessageOrigin {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for LlmMessageOrigin {
    fn graphql_type() -> &'static str {
        "LLMMessageOrigin"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<LlmMessageOrigin> for crate::IdInput<LlmMessageOrigin> {
    fn from(value: LlmMessageOrigin) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<LlmMessageOrigin> for crate::IdInput<super::NodeClient> {
    fn from(value: LlmMessageOrigin) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl LlmMessageOrigin {
    #[doc = "The display name of the sending agent (for AGENT origins) or the observed agent (for EVENT origins).\n\nSelects GraphQL field `agentName` on `LLMMessageOrigin`.\n\n**Experimental:** Agent APIs are likely to change."]
    pub async fn agent_name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("agentName");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this LLMMessageOrigin.\n\nSelects GraphQL field `id` on `LLMMessageOrigin`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Who put this message on the record.\n\nSelects GraphQL field `kind` on `LLMMessageOrigin`.\n\n**Experimental:** Agent APIs are likely to change."]
    pub async fn kind(&self) -> Result<super::LlmMessageOriginKind, crate::QueryError> {
        let query = self.selection.select("kind");
        query.execute(&self.session).await
    }
    #[doc = "The message's short ref within the receiving agent's runtime, e.g. \"#3\": the deterministic token replies name (send's replyTo) and the message lookup takes.\n\nSelects GraphQL field `ref` on `LLMMessageOrigin`.\n\n**Experimental:** Agent APIs are likely to change."]
    pub async fn r#ref(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("ref");
        query.execute(&self.session).await
    }
    #[doc = "The ref of the message this one answers, in the sender's own runtime, if any.\n\nSelects GraphQL field `replyTo` on `LLMMessageOrigin`.\n\n**Experimental:** Agent APIs are likely to change."]
    pub async fn reply_to(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("replyTo");
        query.execute(&self.session).await
    }
}
impl super::Node for LlmMessageOrigin {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
