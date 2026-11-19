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
impl super::ModuleLoadable for Vault {
    fn __module_ref(__query: &dagger_sdk::QueryBuilder, __id: dagger_sdk::IdInput<Self>) -> Self {
        Self::__from_query(__query.generated_lazy_reentry_builder(__id, "Vault"))
    }
    fn __module_load(
        __query: &dagger_sdk::QueryBuilder,
        __id: dagger_sdk::IdInput<Self>,
    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, dagger_sdk::QueryError>>
    + ::core::marker::Send {
        let __query = ::core::clone::Clone::clone(__query);
        async move {
            let __id: dagger_sdk::Id = __query
                .generated_lazy_reentry_builder(__id, "Vault")
                .select("id")
                .execute()
                .await?;
            ::core::result::Result::Ok(Self::__from_query(
                __query.generated_reentry_builder(__id, "Vault"),
            ))
        }
    }
}
impl dagger_sdk::__private::ModuleInputCodec for Vault {
    fn decode_input(
        value: &dagger_sdk::__private::ModuleJson,
        context: &dagger_sdk::__private::ModuleContextBase,
    ) -> ::core::result::Result<Self, dagger_sdk::__private::DecodeError> {
        ::core::result::Result::map(
            dagger_sdk::__private::decode_handle_input(value, context, "Vault"),
            Self::__from_query,
        )
    }
}
impl dagger_sdk::__private::ModuleOutputCodec for Vault {
    fn encode_output<'a>(
        self,
        _context: &'a dagger_sdk::__private::ModuleContextBase,
    ) -> dagger_sdk::__private::ModuleBoxFuture<
        'a,
        ::core::result::Result<
            dagger_sdk::__private::ModuleJson,
            dagger_sdk::__private::EncodeError,
        >,
    >
    where
        Self: 'a,
    {
        dagger_sdk::__private::encode_handle_output(self)
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
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `Query.vault`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct QueryVaultOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument ttl.\n\n`None` omits GraphQL field `ttl` and preserves engine default `Int(60)`."
    )]
    pub ttl: ::core::option::Option<i64>,
}
impl QueryVaultOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `ttl` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_ttl(mut self, value: i64) -> Self {
        self.ttl = ::core::option::Option::Some(value);
        self
    }
}
impl super::ModuleQuery {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field vault.\n\nSelects GraphQL field `vault` on `Query`."
    )]
    #[must_use]
    pub fn vault(
        &self,
        mount: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
        name: impl ::core::convert::Into<::std::string::String>,
    ) -> super::Vault {
        let __query = self.__query_builder().select("vault");
        let __query = __query.generated_argument_id_shape(
            "mount",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(mount),
        );
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        super::Vault::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `vault` with a borrowed, reusable `QueryVaultOpts` value."
    )]
    #[must_use]
    pub fn vault_opts(
        &self,
        mount: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
        name: impl ::core::convert::Into<::std::string::String>,
        opts: &QueryVaultOpts,
    ) -> super::Vault {
        let __query = self.__query_builder().select("vault");
        let __query = __query.generated_argument_id_shape(
            "mount",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(mount),
        );
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        let __query = if let ::core::option::Option::Some(__value) = &opts.ttl {
            __query.argument("ttl", __value)
        } else {
            __query
        };
        super::Vault::__from_query(__query)
    }
}
