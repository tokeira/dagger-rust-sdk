//! Generated bindings owned by the GraphQL `SourceMap` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Source location information."]
#[derive(Clone)]
pub struct SourceMap {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for SourceMap {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for SourceMap {
    fn graphql_type() -> &'static str {
        "SourceMap"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<SourceMap> for crate::IdInput<SourceMap> {
    fn from(value: SourceMap) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<SourceMap> for crate::IdInput<super::NodeClient> {
    fn from(value: SourceMap) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl SourceMap {
    #[doc = "The column number within the line.\n\nSelects GraphQL field `column` on `SourceMap`."]
    pub async fn column(&self) -> Result<i64, crate::QueryError> {
        let query = self.selection.select("column");
        query.execute(&self.session).await
    }
    #[doc = "The filename from the module source.\n\nSelects GraphQL field `filename` on `SourceMap`."]
    pub async fn filename(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("filename");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this SourceMap.\n\nSelects GraphQL field `id` on `SourceMap`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The line number within the filename.\n\nSelects GraphQL field `line` on `SourceMap`."]
    pub async fn line(&self) -> Result<i64, crate::QueryError> {
        let query = self.selection.select("line");
        query.execute(&self.session).await
    }
    #[doc = "The module dependency this was declared in.\n\nSelects GraphQL field `module` on `SourceMap`."]
    pub async fn module(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("module");
        query.execute(&self.session).await
    }
    #[doc = "The URL to the file, if any. This can be used to link to the source map in the browser.\n\nSelects GraphQL field `url` on `SourceMap`."]
    pub async fn url(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("url");
        query.execute(&self.session).await
    }
}
impl super::Node for SourceMap {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
