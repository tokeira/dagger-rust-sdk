//! Generated bindings owned by the GraphQL `GitPushResult` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "A receipt for a completed Git push. Reading or replaying the receipt does not push again."]
#[derive(Clone)]
pub struct GitPushResult {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for GitPushResult {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for GitPushResult {
    fn graphql_type() -> &'static str {
        "GitPushResult"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<GitPushResult> for crate::IdInput<GitPushResult> {
    fn from(value: GitPushResult) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<GitPushResult> for crate::IdInput<super::NodeClient> {
    fn from(value: GitPushResult) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl GitPushResult {
    #[doc = "How the remote ref was updated.\n\nSelects GraphQL field `disposition` on `GitPushResult`."]
    pub async fn disposition(&self) -> Result<super::GitPushDisposition, crate::QueryError> {
        let query = self.selection.select("disposition");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this GitPushResult.\n\nSelects GraphQL field `id` on `GitPushResult`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The previous remote object ID; empty when the ref was created.\n\nSelects GraphQL field `previousSHA` on `GitPushResult`."]
    pub async fn previous_sha(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("previousSHA");
        query.execute(&self.session).await
    }
    #[doc = "The fully qualified remote ref.\n\nSelects GraphQL field `ref` on `GitPushResult`."]
    pub async fn r#ref(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("ref");
        query.execute(&self.session).await
    }
    #[doc = "The object ID pushed to the remote.\n\nSelects GraphQL field `sha` on `GitPushResult`."]
    pub async fn sha(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("sha");
        query.execute(&self.session).await
    }
}
impl super::Node for GitPushResult {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
