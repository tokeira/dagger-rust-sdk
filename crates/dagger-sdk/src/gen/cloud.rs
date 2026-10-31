//! Generated bindings owned by the GraphQL `Cloud` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Dagger Cloud configuration and state"]
#[derive(Clone)]
pub struct Cloud {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for Cloud {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Cloud {
    fn graphql_type() -> &'static str {
        "Cloud"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Cloud> for crate::IdInput<Cloud> {
    fn from(value: Cloud) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Cloud> for crate::IdInput<super::NodeClient> {
    fn from(value: Cloud) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Cloud {
    #[doc = "A unique identifier for this Cloud.\n\nSelects GraphQL field `id` on `Cloud`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The trace URL for the current session\n\nSelects GraphQL field `traceURL` on `Cloud`."]
    pub async fn trace_url(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("traceURL");
        query.execute(&self.session).await
    }
}
impl super::Node for Cloud {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
