//! Generated bindings owned by the GraphQL `SearchSubmatch` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Lazy handle for GraphQL object `SearchSubmatch`."]
#[derive(Clone)]
pub struct SearchSubmatch {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for SearchSubmatch {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for SearchSubmatch {
    fn graphql_type() -> &'static str {
        "SearchSubmatch"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<SearchSubmatch> for crate::IdInput<SearchSubmatch> {
    fn from(value: SearchSubmatch) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<SearchSubmatch> for crate::IdInput<super::NodeClient> {
    fn from(value: SearchSubmatch) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl SearchSubmatch {
    #[doc = "The match's end offset within the matched lines.\n\nSelects GraphQL field `end` on `SearchSubmatch`."]
    pub async fn end(&self) -> Result<i64, crate::QueryError> {
        let query = self.selection.select("end");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this SearchSubmatch.\n\nSelects GraphQL field `id` on `SearchSubmatch`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The match's start offset within the matched lines.\n\nSelects GraphQL field `start` on `SearchSubmatch`."]
    pub async fn start(&self) -> Result<i64, crate::QueryError> {
        let query = self.selection.select("start");
        query.execute(&self.session).await
    }
    #[doc = "The matched text.\n\nSelects GraphQL field `text` on `SearchSubmatch`."]
    pub async fn text(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("text");
        query.execute(&self.session).await
    }
}
impl super::Node for SearchSubmatch {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
