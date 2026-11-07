//! Generated bindings owned by GraphQL coordinate `MinimalNode`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:506c57349967eeaceb1235a4465193229f99ff7b15c2a58a4e8fd04be3d5008f","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
