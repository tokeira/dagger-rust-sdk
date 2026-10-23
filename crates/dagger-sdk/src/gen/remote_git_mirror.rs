//! Generated bindings owned by the GraphQL `RemoteGitMirror` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "An internal persistent bare git mirror."]
#[derive(Clone)]
pub struct RemoteGitMirror {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for RemoteGitMirror {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for RemoteGitMirror {
    fn graphql_type() -> &'static str {
        "RemoteGitMirror"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<RemoteGitMirror> for crate::IdInput<RemoteGitMirror> {
    fn from(value: RemoteGitMirror) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<RemoteGitMirror> for crate::IdInput<super::NodeClient> {
    fn from(value: RemoteGitMirror) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl RemoteGitMirror {
    #[doc = "A unique identifier for this RemoteGitMirror.\n\nSelects GraphQL field `id` on `RemoteGitMirror`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
}
impl super::Node for RemoteGitMirror {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
