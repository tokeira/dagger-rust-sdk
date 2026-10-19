//! Generated bindings owned by the GraphQL `LLMSkill` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "A skill available to a model: task-specific guidance discovered with ListSkills and read with ReadSkill."]
#[derive(Clone)]
pub struct LlmSkill {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for LlmSkill {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for LlmSkill {
    fn graphql_type() -> &'static str {
        "LLMSkill"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<LlmSkill> for crate::IdInput<LlmSkill> {
    fn from(value: LlmSkill) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<LlmSkill> for crate::IdInput<super::NodeClient> {
    fn from(value: LlmSkill) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl LlmSkill {
    #[doc = "The one-line description from the SKILL.md frontmatter.\n\nSelects GraphQL field `description` on `LLMSkill`."]
    pub async fn description(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("description");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this LLMSkill.\n\nSelects GraphQL field `id` on `LLMSkill`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The skill name, as passed to ReadSkill.\n\nSelects GraphQL field `name` on `LLMSkill`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
}
impl super::Node for LlmSkill {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
