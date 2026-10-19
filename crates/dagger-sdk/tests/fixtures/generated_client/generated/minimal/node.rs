//! Generated bindings owned by GraphQL coordinate `MinimalNode`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:cb1ab037106ae8c061b41d518b70d6e25710e8ac697b14d582ed73eea7291882","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Client fixture interface MinimalNode."]
pub trait Node {
    /// Borrows the immutable query represented by this interface handle.
    fn selection(&self) -> &dagger_sdk::QueryBuilder;
}
#[doc = "Lazy handle for GraphQL interface `MinimalNode`."]
#[derive(Clone)]
pub struct NodeClient {
    query: dagger_sdk::QueryBuilder,
}
impl NodeClient {
    #[must_use]
    pub(in crate::dagger_client) fn from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    #[doc = "Client fixture field id."]
    pub async fn id(&self) -> Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let query = self.query.select("id");
        query.execute().await
    }
    #[doc = "Client fixture field message."]
    pub async fn message(&self) -> Result<String, dagger_sdk::QueryError> {
        let query = self.query.select("message");
        query.execute().await
    }
}
impl Node for NodeClient {
    fn selection(&self) -> &dagger_sdk::QueryBuilder {
        &self.query
    }
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for NodeClient {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = Result<dagger_sdk::Id, dagger_sdk::QueryError>>
                + Send,
        >,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl From<NodeClient> for dagger_sdk::IdInput<NodeClient> {
    fn from(value: NodeClient) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
