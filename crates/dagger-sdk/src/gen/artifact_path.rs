//! Generated bindings owned by the GraphQL `ArtifactPath` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "A schema path and its dimensions. The path can exist even when its collections have no runtime items."]
#[derive(Clone)]
pub struct ArtifactPath {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for ArtifactPath {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for ArtifactPath {
    fn graphql_type() -> &'static str {
        "ArtifactPath"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<ArtifactPath> for crate::IdInput<ArtifactPath> {
    fn from(value: ArtifactPath) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<ArtifactPath> for crate::IdInput<super::NodeClient> {
    fn from(value: ArtifactPath) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl ArtifactPath {
    #[doc = "The description of the field at this path.\n\nSelects GraphQL field `description` on `ArtifactPath`."]
    pub async fn description(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("description");
        query.execute(&self.session).await
    }
    #[doc = "The dimension identifiers required by this path.\n\nSelects GraphQL field `dimensions` on `ArtifactPath`."]
    pub async fn dimensions(&self) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("dimensions");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this ArtifactPath.\n\nSelects GraphQL field `id` on `ArtifactPath`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "A module load failure for this path, or an empty string.\n\nSelects GraphQL field `loadError` on `ArtifactPath`."]
    pub async fn load_error(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("loadError");
        query.execute(&self.session).await
    }
    #[doc = "The installed module name.\n\nSelects GraphQL field `moduleName` on `ArtifactPath`."]
    pub async fn module_name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("moduleName");
        query.execute(&self.session).await
    }
    #[doc = "The DAG address of this path, without dimension keys.\n\nSelects GraphQL field `uri` on `ArtifactPath`."]
    pub async fn uri(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("uri");
        query.execute(&self.session).await
    }
}
impl super::Node for ArtifactPath {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
