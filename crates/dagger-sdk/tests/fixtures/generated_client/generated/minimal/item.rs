//! Generated bindings owned by GraphQL coordinate `MinimalItem`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:506c57349967eeaceb1235a4465193229f99ff7b15c2a58a4e8fd04be3d5008f","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
