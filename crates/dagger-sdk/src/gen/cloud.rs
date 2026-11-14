//! Generated bindings owned by the GraphQL `Cloud` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
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
    #[doc = "The trace URL for the current session\n\nSelects GraphQL field `traceUrl` on `Cloud`."]
    pub async fn trace_url(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("traceUrl");
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
