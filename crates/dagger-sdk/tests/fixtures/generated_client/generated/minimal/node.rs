//! Generated bindings owned by GraphQL coordinate `MinimalNode`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
