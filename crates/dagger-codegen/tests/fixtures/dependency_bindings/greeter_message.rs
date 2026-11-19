//! Bindings owned by visible GraphQL type `GreeterMessage`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture object GreeterMessage.")]
#[derive(Clone)]
pub struct GreeterMessage {
    query: dagger_sdk::QueryBuilder,
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `GreeterMessage.describe`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct GreeterMessageDescribeOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument prefix.\n\n`None` omits GraphQL field `prefix` and preserves engine default `String(\"\")`."
    )]
    pub prefix: ::core::option::Option<::std::string::String>,
}
impl GreeterMessageDescribeOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `prefix` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_prefix(mut self, value: impl ::core::convert::Into<::std::string::String>) -> Self {
        self.prefix = ::core::option::Option::Some(::core::convert::Into::into(value));
        self
    }
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `GreeterMessage.relabel`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct GreeterMessageRelabelOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument loud.\n\n`None` omits GraphQL field `loud` and preserves engine default `Boolean(false)`."
    )]
    pub loud: ::core::option::Option<bool>,
}
impl GreeterMessageRelabelOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `loud` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_loud(mut self, value: bool) -> Self {
        self.loud = ::core::option::Option::Some(value);
        self
    }
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for GreeterMessage {
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
impl ::core::convert::From<GreeterMessage> for dagger_sdk::IdInput<GreeterMessage> {
    fn from(value: GreeterMessage) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl ::core::convert::From<GreeterMessage> for dagger_sdk::IdInput<super::GreeterNamedClient> {
    fn from(value: GreeterMessage) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl ::core::convert::From<GreeterMessage> for dagger_sdk::IdInput<dagger_sdk::NodeClient> {
    fn from(value: GreeterMessage) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl super::ModuleLoadable for GreeterMessage {
    fn __module_ref(__query: &dagger_sdk::QueryBuilder, __id: dagger_sdk::IdInput<Self>) -> Self {
        Self::__from_query(__query.generated_lazy_reentry_builder(__id, "GreeterMessage"))
    }
    fn __module_load(
        __query: &dagger_sdk::QueryBuilder,
        __id: dagger_sdk::IdInput<Self>,
    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, dagger_sdk::QueryError>>
    + ::core::marker::Send {
        let __query = ::core::clone::Clone::clone(__query);
        async move {
            let __id: dagger_sdk::Id = __query
                .generated_lazy_reentry_builder(__id, "GreeterMessage")
                .select("id")
                .execute()
                .await?;
            ::core::result::Result::Ok(Self::__from_query(
                __query.generated_reentry_builder(__id, "GreeterMessage"),
            ))
        }
    }
}
impl dagger_sdk::__private::ModuleInputCodec for GreeterMessage {
    fn decode_input(
        value: &dagger_sdk::__private::ModuleJson,
        context: &dagger_sdk::__private::ModuleContextBase,
    ) -> ::core::result::Result<Self, dagger_sdk::__private::DecodeError> {
        ::core::result::Result::map(
            dagger_sdk::__private::decode_handle_input(value, context, "GreeterMessage"),
            Self::__from_query,
        )
    }
}
impl dagger_sdk::__private::ModuleOutputCodec for GreeterMessage {
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
impl GreeterMessage {
    #[doc(hidden)]
    pub fn __from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field aliases.\n\nSelects GraphQL field `aliases` on `GreeterMessage`."
    )]
    pub async fn aliases(
        &self,
    ) -> ::core::result::Result<::std::vec::Vec<super::GreeterNamedClient>, dagger_sdk::QueryError>
    {
        let __query = self.query.select("aliases");
        let __ids: ::std::vec::Vec<dagger_sdk::Id> = __query.select("id").execute().await?;
        ::core::result::Result::Ok(::core::iter::Iterator::collect(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(__ids),
                |__value_0| {
                    super::GreeterNamedClient::__from_query(
                        __query.generated_reentry_builder(__value_0, "GreeterNamed"),
                    )
                },
            ),
        ))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field describe.\n\nSelects GraphQL field `describe` on `GreeterMessage`."
    )]
    pub async fn describe(
        &self,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("describe");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `describe` with a borrowed, reusable `GreeterMessageDescribeOpts` value."
    )]
    pub async fn describe_opts(
        &self,
        opts: &GreeterMessageDescribeOpts,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("describe");
        let __query = if let ::core::option::Option::Some(__value) = &opts.prefix {
            __query.argument("prefix", __value)
        } else {
            __query
        };
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field id.\n\nSelects GraphQL field `id` on `GreeterMessage`."
    )]
    pub async fn id(&self) -> ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let __query = self.query.select("id");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field message.\n\nSelects GraphQL field `message` on `GreeterMessage`."
    )]
    pub async fn message(
        &self,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("message");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field mode.\n\nSelects GraphQL field `mode` on `GreeterMessage`."
    )]
    pub async fn mode(&self) -> ::core::result::Result<super::GreeterMode, dagger_sdk::QueryError> {
        let __query = self.query.select("mode");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field relabel.\n\nSelects GraphQL field `relabel` on `GreeterMessage`."
    )]
    pub async fn relabel(
        &self,
        label: impl ::core::convert::Into<::std::string::String>,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
        tags: ::std::vec::Vec<impl ::core::convert::Into<::std::string::String>>,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("relabel");
        let __query = __query.argument(
            "label",
            ::core::convert::Into::<::std::string::String>::into(label),
        );
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        let tags = ::core::iter::Iterator::collect::<::std::vec::Vec<::std::string::String>>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(tags),
                ::core::convert::Into::into,
            ),
        );
        let __query = __query.argument("tags", tags);
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `relabel` with a borrowed, reusable `GreeterMessageRelabelOpts` value."
    )]
    pub async fn relabel_opts(
        &self,
        label: impl ::core::convert::Into<::std::string::String>,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
        tags: ::std::vec::Vec<impl ::core::convert::Into<::std::string::String>>,
        opts: &GreeterMessageRelabelOpts,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("relabel");
        let __query = __query.argument(
            "label",
            ::core::convert::Into::<::std::string::String>::into(label),
        );
        let __query = if let ::core::option::Option::Some(__value) = &opts.loud {
            __query.argument("loud", __value)
        } else {
            __query
        };
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        let tags = ::core::iter::Iterator::collect::<::std::vec::Vec<::std::string::String>>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(tags),
                ::core::convert::Into::into,
            ),
        );
        let __query = __query.argument("tags", tags);
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field renamed.\n\nSelects GraphQL field `renamed` on `GreeterMessage`."
    )]
    #[must_use]
    pub fn renamed(
        &self,
        name: impl ::core::convert::Into<::std::string::String>,
    ) -> super::GreeterNamedClient {
        let __query = self.query.select("renamed");
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        super::GreeterNamedClient::__from_query(__query)
    }
}
impl super::GreeterNamed for GreeterMessage {
    fn aliases(
        &self,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<
            ::std::vec::Vec<super::GreeterNamedClient>,
            dagger_sdk::QueryError,
        >,
    > + ::core::marker::Send {
        let __query = self.query.select("aliases");
        async move {
            let __ids: ::std::vec::Vec<dagger_sdk::Id> = __query.select("id").execute().await?;
            ::core::result::Result::Ok(::core::iter::Iterator::collect(
                ::core::iter::Iterator::map(
                    ::core::iter::IntoIterator::into_iter(__ids),
                    |__value_0| {
                        super::GreeterNamedClient::__from_query(
                            __query.generated_reentry_builder(__value_0, "GreeterNamed"),
                        )
                    },
                ),
            ))
        }
    }
    fn describe(
        &self,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<::std::string::String, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("describe");
        async move { __query.execute().await }
    }
    fn describe_opts(
        &self,
        opts: &super::GreeterNamedDescribeOpts,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<::std::string::String, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("describe");
        let __query = if let ::core::option::Option::Some(__value) = &opts.prefix {
            __query.argument("prefix", __value)
        } else {
            __query
        };
        async move { __query.execute().await }
    }
    fn id(
        &self,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("id");
        async move { __query.execute().await }
    }
    fn message(
        &self,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<::std::string::String, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("message");
        async move { __query.execute().await }
    }
    fn relabel(
        &self,
        label: impl ::core::convert::Into<::std::string::String> + ::core::marker::Send,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>
        + ::core::marker::Send,
        tags: ::std::vec::Vec<
            impl ::core::convert::Into<::std::string::String> + ::core::marker::Send,
        >,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<::std::string::String, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("relabel");
        let __query = __query.argument(
            "label",
            ::core::convert::Into::<::std::string::String>::into(label),
        );
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        let tags = ::core::iter::Iterator::collect::<::std::vec::Vec<::std::string::String>>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(tags),
                ::core::convert::Into::into,
            ),
        );
        let __query = __query.argument("tags", tags);
        async move { __query.execute().await }
    }
    fn relabel_opts(
        &self,
        label: impl ::core::convert::Into<::std::string::String> + ::core::marker::Send,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>
        + ::core::marker::Send,
        tags: ::std::vec::Vec<
            impl ::core::convert::Into<::std::string::String> + ::core::marker::Send,
        >,
        opts: &super::GreeterNamedRelabelOpts,
    ) -> impl ::core::future::Future<
        Output = ::core::result::Result<::std::string::String, dagger_sdk::QueryError>,
    > + ::core::marker::Send {
        let __query = self.query.select("relabel");
        let __query = __query.argument(
            "label",
            ::core::convert::Into::<::std::string::String>::into(label),
        );
        let __query = if let ::core::option::Option::Some(__value) = &opts.loud {
            __query.argument("loud", __value)
        } else {
            __query
        };
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        let tags = ::core::iter::Iterator::collect::<::std::vec::Vec<::std::string::String>>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(tags),
                ::core::convert::Into::into,
            ),
        );
        let __query = __query.argument("tags", tags);
        async move { __query.execute().await }
    }
    fn renamed(
        &self,
        name: impl ::core::convert::Into<::std::string::String> + ::core::marker::Send,
    ) -> super::GreeterNamedClient {
        let __query = self.query.select("renamed");
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        super::GreeterNamedClient::__from_query(__query)
    }
}
