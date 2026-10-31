//! Generated bindings owned by the GraphQL `ModuleConfigClient` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "The client generated for the module."]
#[derive(Clone)]
pub struct ModuleConfigClient {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for ModuleConfigClient {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for ModuleConfigClient {
    fn graphql_type() -> &'static str {
        "ModuleConfigClient"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<ModuleConfigClient> for crate::IdInput<ModuleConfigClient> {
    fn from(value: ModuleConfigClient) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<ModuleConfigClient> for crate::IdInput<super::NodeClient> {
    fn from(value: ModuleConfigClient) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl ModuleConfigClient {
    #[doc = "The directory the client is generated in.\n\nSelects GraphQL field `directory` on `ModuleConfigClient`."]
    pub async fn directory(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("directory");
        query.execute(&self.session).await
    }
    #[doc = "The generator to use\n\nSelects GraphQL field `generator` on `ModuleConfigClient`."]
    pub async fn generator(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("generator");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this ModuleConfigClient.\n\nSelects GraphQL field `id` on `ModuleConfigClient`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
}
impl super::Node for ModuleConfigClient {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
