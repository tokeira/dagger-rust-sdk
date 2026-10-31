//! Generated bindings owned by the GraphQL `IdentifierWord` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "One word of an identifier."]
#[derive(Clone)]
pub struct IdentifierWord {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for IdentifierWord {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for IdentifierWord {
    fn graphql_type() -> &'static str {
        "IdentifierWord"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<IdentifierWord> for crate::IdInput<IdentifierWord> {
    fn from(value: IdentifierWord) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<IdentifierWord> for crate::IdInput<super::NodeClient> {
    fn from(value: IdentifierWord) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl IdentifierWord {
    #[doc = "A unique identifier for this IdentifierWord.\n\nSelects GraphQL field `id` on `IdentifierWord`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The kind of word.\n\nSelects GraphQL field `kind` on `IdentifierWord`."]
    pub async fn kind(&self) -> Result<super::IdentifierWordKind, crate::QueryError> {
        let query = self.selection.select("kind");
        query.execute(&self.session).await
    }
    #[doc = "Trailing digits and/or a plural \"s\", in that order: SHA+\"s\", OAuth+\"2\", HTTP+\"2s\".\n\nSelects GraphQL field `suffix` on `IdentifierWord`."]
    pub async fn suffix(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("suffix");
        query.execute(&self.session).await
    }
    #[doc = "The dictionary entry this word matched, if any.\n\nSelects GraphQL field `term` on `IdentifierWord`."]
    pub async fn term(&self) -> Result<Option<super::NamingTerm>, crate::QueryError> {
        let query = self.selection.select("term");
        let query = query.select("id");
        query
            .execute_reentry::<super::NamingTerm, Option<crate::Id>>(&self.session, "NamingTerm")
            .await
    }
    #[doc = "Standard spelling: \"client\" (WORD), \"HTTP\" (ACRONYM), \"GitHub\" (TERM).\n\nSelects GraphQL field `text` on `IdentifierWord`."]
    pub async fn text(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("text");
        query.execute(&self.session).await
    }
}
impl super::Node for IdentifierWord {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
