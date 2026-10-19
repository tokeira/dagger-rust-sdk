//! Generated bindings owned by the GraphQL `RemoteGitMirror` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
