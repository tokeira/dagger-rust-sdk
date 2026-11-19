//! Bindings owned by visible GraphQL type `Clock`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture object Clock.")]
#[derive(Clone)]
pub struct Clock {
    query: dagger_sdk::QueryBuilder,
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for Clock {
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
impl ::core::convert::From<Clock> for dagger_sdk::IdInput<Clock> {
    fn from(value: Clock) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl ::core::convert::From<Clock> for dagger_sdk::IdInput<dagger_sdk::NodeClient> {
    fn from(value: Clock) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl super::ModuleLoadable for Clock {
    fn __module_ref(__query: &dagger_sdk::QueryBuilder, __id: dagger_sdk::IdInput<Self>) -> Self {
        Self::__from_query(__query.generated_lazy_reentry_builder(__id, "Clock"))
    }
    fn __module_load(
        __query: &dagger_sdk::QueryBuilder,
        __id: dagger_sdk::IdInput<Self>,
    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, dagger_sdk::QueryError>>
    + ::core::marker::Send {
        let __query = ::core::clone::Clone::clone(__query);
        async move {
            let __id: dagger_sdk::Id = __query
                .generated_lazy_reentry_builder(__id, "Clock")
                .select("id")
                .execute()
                .await?;
            ::core::result::Result::Ok(Self::__from_query(
                __query.generated_reentry_builder(__id, "Clock"),
            ))
        }
    }
}
impl Clock {
    #[doc(hidden)]
    pub fn __from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field id.\n\nSelects GraphQL field `id` on `Clock`."
    )]
    pub async fn id(&self) -> ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let __query = self.query.select("id");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field now.\n\nSelects GraphQL field `now` on `Clock`."
    )]
    pub async fn now(
        &self,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("now");
        __query.execute().await
    }
}
impl super::ModuleQuery {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field clock.\n\nSelects GraphQL field `clock` on `Query`."
    )]
    #[must_use]
    pub fn clock(&self) -> super::Clock {
        let __query = self.__query_builder().select("clock");
        super::Clock::__from_query(__query)
    }
}
