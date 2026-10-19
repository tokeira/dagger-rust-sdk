//! Generated bindings owned by the GraphQL `ErrorValue` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Lazy handle for GraphQL object `ErrorValue`."]
#[derive(Clone)]
pub struct ErrorValue {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for ErrorValue {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for ErrorValue {
    fn graphql_type() -> &'static str {
        "ErrorValue"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<ErrorValue> for crate::IdInput<ErrorValue> {
    fn from(value: ErrorValue) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<ErrorValue> for crate::IdInput<super::NodeClient> {
    fn from(value: ErrorValue) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl ErrorValue {
    #[doc = "A unique identifier for this ErrorValue.\n\nSelects GraphQL field `id` on `ErrorValue`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The name of the value.\n\nSelects GraphQL field `name` on `ErrorValue`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
    #[doc = "The value.\n\nSelects GraphQL field `value` on `ErrorValue`."]
    pub async fn value(&self) -> Result<crate::Json, crate::QueryError> {
        let query = self.selection.select("value");
        query.execute(&self.session).await
    }
}
impl super::Node for ErrorValue {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
