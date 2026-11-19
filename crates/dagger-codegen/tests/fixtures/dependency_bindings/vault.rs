//! Bindings owned by visible GraphQL type `Vault`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture object Vault.")]
#[derive(Clone)]
pub struct Vault {
    query: dagger_sdk::QueryBuilder,
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for Vault {
    fn into_id(
        self,
    ) -> ::core::pin::Pin<
        ::std::boxed::Box<
            dyn ::core::future::Future<
                    Output = ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError>,
                > + ::core::marker::Send,
        >,
    > {
        ::std::boxed::Box::pin(async move { self.id().await })
    }
}
impl ::core::convert::From<Vault> for dagger_sdk::IdInput<Vault> {
    fn from(value: Vault) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl ::core::convert::From<Vault> for dagger_sdk::IdInput<dagger_sdk::NodeClient> {
    fn from(value: Vault) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl Vault {
    #[doc(hidden)]
    pub fn __from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field id.\n\nSelects GraphQL field `id` on `Vault`."
    )]
    pub async fn id(&self) -> ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let __query = self.query.select("id");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field name.\n\nSelects GraphQL field `name` on `Vault`."
    )]
    pub async fn name(
        &self,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("name");
        __query.execute().await
    }
}
