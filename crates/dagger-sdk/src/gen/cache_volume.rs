//! Generated bindings owned by the GraphQL `CacheVolume` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "A directory whose contents persist across runs."]
#[derive(Clone)]
pub struct CacheVolume {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for CacheVolume {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for CacheVolume {
    fn graphql_type() -> &'static str {
        "CacheVolume"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<CacheVolume> for crate::IdInput<CacheVolume> {
    fn from(value: CacheVolume) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<CacheVolume> for crate::IdInput<super::NodeClient> {
    fn from(value: CacheVolume) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl CacheVolume {
    #[doc = "A unique identifier for this CacheVolume.\n\nSelects GraphQL field `id` on `CacheVolume`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
}
impl super::Node for CacheVolume {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
