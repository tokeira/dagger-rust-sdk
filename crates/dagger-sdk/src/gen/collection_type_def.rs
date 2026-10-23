//! Generated bindings owned by the GraphQL `CollectionTypeDef` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Lazy handle for GraphQL object `CollectionTypeDef`."]
#[derive(Clone)]
pub struct CollectionTypeDef {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for CollectionTypeDef {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for CollectionTypeDef {
    fn graphql_type() -> &'static str {
        "CollectionTypeDef"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<CollectionTypeDef> for crate::IdInput<CollectionTypeDef> {
    fn from(value: CollectionTypeDef) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<CollectionTypeDef> for crate::IdInput<super::NodeClient> {
    fn from(value: CollectionTypeDef) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl CollectionTypeDef {
    #[doc = "The type of batch operations, or null when there are none.\n\nSelects GraphQL field `batchType` on `CollectionTypeDef`."]
    pub async fn batch_type(&self) -> Result<Option<super::TypeDef>, crate::QueryError> {
        let query = self.selection.select("batchType");
        let query = query.select("id");
        query
            .execute_reentry::<super::TypeDef, Option<crate::Id>>(&self.session, "TypeDef")
            .await
    }
    #[doc = "A unique identifier for this CollectionTypeDef.\n\nSelects GraphQL field `id` on `CollectionTypeDef`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The type of collection keys.\n\nSelects GraphQL field `keyType` on `CollectionTypeDef`."]
    #[must_use]
    pub fn key_type(&self) -> super::TypeDef {
        let query = self.selection.select("keyType");
        super::TypeDef {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "The object type returned by get.\n\nSelects GraphQL field `valueType` on `CollectionTypeDef`."]
    #[must_use]
    pub fn value_type(&self) -> super::TypeDef {
        let query = self.selection.select("valueType");
        super::TypeDef {
            session: self.session.clone(),
            selection: query,
        }
    }
}
impl super::Node for CollectionTypeDef {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
