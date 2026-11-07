//! Generated bindings owned by the GraphQL `RemoteGitMirror` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
