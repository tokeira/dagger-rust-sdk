//! Generated bindings owned by the GraphQL `Terminal` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
