//! Generated bindings owned by the GraphQL `GitRemote` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "A named reference to a remote Git repository."]
#[derive(Clone)]
pub struct GitRemote {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for GitRemote {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for GitRemote {
    fn graphql_type() -> &'static str {
        "GitRemote"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<GitRemote> for crate::IdInput<GitRemote> {
    fn from(value: GitRemote) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<GitRemote> for crate::IdInput<super::NodeClient> {
    fn from(value: GitRemote) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl GitRemote {
    #[doc = "A unique identifier for this GitRemote.\n\nSelects GraphQL field `id` on `GitRemote`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The remote's name.\n\nSelects GraphQL field `name` on `GitRemote`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
    #[doc = "Access this remote's repository using its fetch URL and the caller's credentials, or the source's existing capability for this exact destination.\n\nHEAD is the remote's HEAD, independent of the workspace's selected commit. Remote registration alone does not grant credentials.\n\nSelects GraphQL field `repository` on `GitRemote`."]
    #[must_use]
    pub fn repository(&self) -> super::GitRepository {
        let query = self.selection.select("repository");
        super::GitRepository {
            session: self.session.clone(),
            selection: query,
        }
    }
}
impl super::Node for GitRemote {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
