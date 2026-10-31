//! Generated bindings owned by the GraphQL `RemoteGitMirror` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
