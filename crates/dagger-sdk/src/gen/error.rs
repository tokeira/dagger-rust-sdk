//! Generated bindings owned by the GraphQL `Error` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Lazy handle for GraphQL object `Error`."]
#[derive(Clone)]
pub struct Error {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for Error {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Error {
    fn graphql_type() -> &'static str {
        "Error"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Error> for crate::IdInput<Error> {
    fn from(value: Error) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Error> for crate::IdInput<super::NodeClient> {
    fn from(value: Error) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Error {
    #[doc = "A unique identifier for this Error.\n\nSelects GraphQL field `id` on `Error`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "A description of the error.\n\nSelects GraphQL field `message` on `Error`."]
    pub async fn message(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("message");
        query.execute(&self.session).await
    }
    #[doc = "The extensions of the error.\n\nSelects GraphQL field `values` on `Error`."]
    pub async fn values(&self) -> Result<Vec<super::ErrorValue>, crate::QueryError> {
        let query = self.selection.select("values");
        let query = query.select("id");
        query
            .execute_reentry::<super::ErrorValue, Vec<crate::Id>>(&self.session, "ErrorValue")
            .await
    }
    #[doc = "Add a value to the error.\n\nSelects GraphQL field `withValue` on `Error`."]
    #[must_use]
    pub fn with_value(&self, name: impl Into<String>, value: crate::Json) -> super::Error {
        let query = self.selection.select("withValue");
        let query = query.arg("name", name.into());
        let query = query.arg("value", value);
        super::Error {
            session: self.session.clone(),
            selection: query,
        }
    }
}
impl super::Node for Error {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
