//! Generated bindings owned by GraphQL coordinate `MinimalItem`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Client fixture object MinimalItem."]
#[derive(Clone)]
pub struct Item {
    query: dagger_sdk::QueryBuilder,
}
impl Item {
    #[must_use]
    pub(in crate::dagger_client) fn from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    /// Borrows the immutable query represented by this handle.
    #[must_use]
    pub fn selection(&self) -> &dagger_sdk::QueryBuilder {
        &self.query
    }
    #[doc = "Client fixture field id."]
    pub async fn id(&self) -> Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let query = self.query.select("id");
        query.execute().await
    }
    #[doc = "Client fixture field state."]
    pub async fn state(&self) -> Result<super::State, dagger_sdk::QueryError> {
        let query = self.query.select("state");
        query.execute().await
    }
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for Item {
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
impl From<Item> for dagger_sdk::IdInput<Item> {
    fn from(value: Item) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl super::Node for Item {
    fn selection(&self) -> &dagger_sdk::QueryBuilder {
        &self.query
    }
}
