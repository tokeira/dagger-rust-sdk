//! Generated bindings owned by the GraphQL `Terminal` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "An interactive terminal that clients can connect to."]
#[derive(Clone)]
pub struct Terminal {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for Terminal {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Terminal {
    fn graphql_type() -> &'static str {
        "Terminal"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Terminal> for crate::IdInput<Terminal> {
    fn from(value: Terminal) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Terminal> for crate::IdInput<super::NodeClient> {
    fn from(value: Terminal) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Terminal> for crate::IdInput<super::SyncerClient> {
    fn from(value: Terminal) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Terminal {
    #[doc = "A unique identifier for this Terminal.\n\nSelects GraphQL field `id` on `Terminal`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Forces evaluation of the pipeline in the engine.\n\nIt doesn't run the default command if no exec has been set.\n\nSelects GraphQL field `sync` on `Terminal`."]
    pub async fn sync(&self) -> Result<super::Terminal, crate::QueryError> {
        let query = self.selection.select("sync");
        let id: crate::Id = query.execute(&self.session).await?;
        Ok(crate::query::reenter::<super::Terminal>(
            &self.session,
            id,
            "Terminal",
        ))
    }
}
impl super::Node for Terminal {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
impl super::Syncer for Terminal {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
    fn sync(
        &self,
    ) -> impl core::future::Future<Output = Result<super::SyncerClient, crate::QueryError>> + Send
    {
        let query = self.selection.select("sync");
        let session = self.session.clone();
        async move {
            let id: crate::Id = query.execute(&session).await?;
            Ok(crate::query::reenter::<super::SyncerClient>(
                &session, id, "Syncer",
            ))
        }
    }
}
