//! Bindings owned by visible GraphQL type `Greeter`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture object Greeter.")]
#[derive(Clone)]
pub struct Greeter {
    query: dagger_sdk::QueryBuilder,
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `Greeter.container`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct GreeterContainerOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument base.\n\n`None` omits GraphQL field `base`."
    )]
    pub base: ::core::option::Option<dagger_sdk::IdInput<dagger_sdk::Container>>,
}
impl GreeterContainerOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `base` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_base(mut self, value: dagger_sdk::IdInput<dagger_sdk::Container>) -> Self {
        self.base = ::core::option::Option::Some(value);
        self
    }
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `Greeter.platform`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct GreeterPlatformOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument platform.\n\n`None` omits GraphQL field `platform`."
    )]
    pub platform: ::core::option::Option<dagger_sdk::Platform>,
}
impl GreeterPlatformOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `platform` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_platform(mut self, value: dagger_sdk::Platform) -> Self {
        self.platform = ::core::option::Option::Some(value);
        self
    }
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `Greeter.shout`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct GreeterShoutOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument mode.\n\n`None` omits GraphQL field `mode`."
    )]
    pub mode: ::core::option::Option<super::GreeterMode>,
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument suffixes.\n\n`None` omits GraphQL field `suffixes`."
    )]
    pub suffixes: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument times.\n\n`None` omits GraphQL field `times` and preserves engine default `Int(1)`."
    )]
    pub times: ::core::option::Option<i64>,
}
impl GreeterShoutOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `mode` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_mode(mut self, value: super::GreeterMode) -> Self {
        self.mode = ::core::option::Option::Some(value);
        self
    }
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `suffixes` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_suffixes(
        mut self,
        value: ::std::vec::Vec<impl ::core::convert::Into<::std::string::String>>,
    ) -> Self {
        self.suffixes = ::core::option::Option::Some(::core::iter::Iterator::collect(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(value),
                ::core::convert::Into::into,
            ),
        ));
        self
    }
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `times` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_times(mut self, value: i64) -> Self {
        self.times = ::core::option::Option::Some(value);
        self
    }
}
impl dagger_sdk::IntoID<dagger_sdk::Id> for Greeter {
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
impl ::core::convert::From<Greeter> for dagger_sdk::IdInput<Greeter> {
    fn from(value: Greeter) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl ::core::convert::From<Greeter> for dagger_sdk::IdInput<dagger_sdk::NodeClient> {
    fn from(value: Greeter) -> Self {
        dagger_sdk::IdInput::generated_lazy(value)
    }
}
impl super::ModuleLoadable for Greeter {
    fn __module_ref(__query: &dagger_sdk::QueryBuilder, __id: dagger_sdk::IdInput<Self>) -> Self {
        Self::__from_query(__query.generated_lazy_reentry_builder(__id, "Greeter"))
    }
    fn __module_load(
        __query: &dagger_sdk::QueryBuilder,
        __id: dagger_sdk::IdInput<Self>,
    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, dagger_sdk::QueryError>>
    + ::core::marker::Send {
        let __query = ::core::clone::Clone::clone(__query);
        async move {
            let __id: dagger_sdk::Id = __query
                .generated_lazy_reentry_builder(__id, "Greeter")
                .select("id")
                .execute()
                .await?;
            ::core::result::Result::Ok(Self::__from_query(
                __query.generated_reentry_builder(__id, "Greeter"),
            ))
        }
    }
}
impl Greeter {
    #[doc(hidden)]
    pub fn __from_query(query: dagger_sdk::QueryBuilder) -> Self {
        Self { query }
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field container.\n\nSelects GraphQL field `container` on `Greeter`."
    )]
    #[must_use]
    pub fn container(
        &self,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
    ) -> dagger_sdk::Container {
        let __query = self.query.select("container");
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        __query.generated_core_handle::<dagger_sdk::Container>()
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `container` with a borrowed, reusable `GreeterContainerOpts` value."
    )]
    #[must_use]
    pub fn container_opts(
        &self,
        source: impl ::core::convert::Into<dagger_sdk::IdInput<dagger_sdk::Directory>>,
        opts: &GreeterContainerOpts,
    ) -> dagger_sdk::Container {
        let __query = self.query.select("container");
        let __query = if let ::core::option::Option::Some(__value) = &opts.base {
            __query.generated_argument_id_shape("base", ::core::clone::Clone::clone(__value))
        } else {
            __query
        };
        let __query = __query.generated_argument_id_shape(
            "source",
            ::core::convert::Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source),
        );
        __query.generated_core_handle::<dagger_sdk::Container>()
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field describe.\n\nSelects GraphQL field `describe` on `Greeter`."
    )]
    pub async fn describe(
        &self,
        named: impl ::core::convert::Into<dagger_sdk::IdInput<super::GreeterNamedClient>>,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("describe");
        let __query = __query.generated_argument_id_shape(
            "named",
            ::core::convert::Into::<dagger_sdk::IdInput<super::GreeterNamedClient>>::into(named),
        );
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field files.\n\nSelects GraphQL field `files` on `Greeter`."
    )]
    pub async fn files(
        &self,
    ) -> ::core::result::Result<::std::vec::Vec<dagger_sdk::File>, dagger_sdk::QueryError> {
        let __query = self.query.select("files");
        let __ids: ::std::vec::Vec<dagger_sdk::Id> = __query.select("id").execute().await?;
        ::core::result::Result::Ok(::core::iter::Iterator::collect(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(__ids),
                |__value_0| {
                    __query
                        .generated_reentry_builder(__value_0, "File")
                        .generated_core_handle::<dagger_sdk::File>()
                },
            ),
        ))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field hello.\n\nSelects GraphQL field `hello` on `Greeter`."
    )]
    pub async fn hello(
        &self,
        name: impl ::core::convert::Into<::std::string::String>,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("hello");
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field id.\n\nSelects GraphQL field `id` on `Greeter`."
    )]
    pub async fn id(&self) -> ::core::result::Result<dagger_sdk::Id, dagger_sdk::QueryError> {
        let __query = self.query.select("id");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field lastMessage.\n\nSelects GraphQL field `lastMessage` on `Greeter`."
    )]
    pub async fn last_message(
        &self,
    ) -> ::core::result::Result<::core::option::Option<super::GreeterMessage>, dagger_sdk::QueryError>
    {
        let __query = self.query.select("lastMessage");
        let __ids: ::core::option::Option<dagger_sdk::Id> = __query.select("id").execute().await?;
        ::core::result::Result::Ok(::core::option::Option::map(__ids, |__value_0| {
            super::GreeterMessage::__from_query(
                __query.generated_reentry_builder(__value_0, "GreeterMessage"),
            )
        }))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field maybeFile.\n\nSelects GraphQL field `maybeFile` on `Greeter`."
    )]
    pub async fn maybe_file(
        &self,
    ) -> ::core::result::Result<::core::option::Option<dagger_sdk::File>, dagger_sdk::QueryError>
    {
        let __query = self.query.select("maybeFile");
        let __ids: ::core::option::Option<dagger_sdk::Id> = __query.select("id").execute().await?;
        ::core::result::Result::Ok(::core::option::Option::map(__ids, |__value_0| {
            __query
                .generated_reentry_builder(__value_0, "File")
                .generated_core_handle::<dagger_sdk::File>()
        }))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field message.\n\nSelects GraphQL field `message` on `Greeter`."
    )]
    #[must_use]
    pub fn message(
        &self,
        name: impl ::core::convert::Into<::std::string::String>,
    ) -> super::GreeterMessage {
        let __query = self.query.select("message");
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        super::GreeterMessage::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field messages.\n\nSelects GraphQL field `messages` on `Greeter`."
    )]
    pub async fn messages(
        &self,
        names: ::std::vec::Vec<impl ::core::convert::Into<::std::string::String>>,
    ) -> ::core::result::Result<::std::vec::Vec<super::GreeterMessage>, dagger_sdk::QueryError>
    {
        let __query = self.query.select("messages");
        let names = ::core::iter::Iterator::collect::<::std::vec::Vec<::std::string::String>>(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(names),
                ::core::convert::Into::into,
            ),
        );
        let __query = __query.argument("names", names);
        let __ids: ::std::vec::Vec<dagger_sdk::Id> = __query.select("id").execute().await?;
        ::core::result::Result::Ok(::core::iter::Iterator::collect(
            ::core::iter::Iterator::map(
                ::core::iter::IntoIterator::into_iter(__ids),
                |__value_0| {
                    super::GreeterMessage::__from_query(
                        __query.generated_reentry_builder(__value_0, "GreeterMessage"),
                    )
                },
            ),
        ))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field mode.\n\nSelects GraphQL field `mode` on `Greeter`."
    )]
    pub async fn mode(&self) -> ::core::result::Result<super::GreeterMode, dagger_sdk::QueryError> {
        let __query = self.query.select("mode");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field named.\n\nSelects GraphQL field `named` on `Greeter`."
    )]
    #[must_use]
    pub fn named(&self) -> super::GreeterNamedClient {
        let __query = self.query.select("named");
        super::GreeterNamedClient::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field platform.\n\nSelects GraphQL field `platform` on `Greeter`."
    )]
    pub async fn platform(
        &self,
    ) -> ::core::result::Result<dagger_sdk::Platform, dagger_sdk::QueryError> {
        let __query = self.query.select("platform");
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `platform` with a borrowed, reusable `GreeterPlatformOpts` value."
    )]
    pub async fn platform_opts(
        &self,
        opts: &GreeterPlatformOpts,
    ) -> ::core::result::Result<dagger_sdk::Platform, dagger_sdk::QueryError> {
        let __query = self.query.select("platform");
        let __query = if let ::core::option::Option::Some(__value) = &opts.platform {
            __query.argument("platform", __value)
        } else {
            __query
        };
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field protocol.\n\nSelects GraphQL field `protocol` on `Greeter`."
    )]
    pub async fn protocol(
        &self,
        protocol: dagger_sdk::NetworkProtocol,
    ) -> ::core::result::Result<dagger_sdk::NetworkProtocol, dagger_sdk::QueryError> {
        let __query = self.query.select("protocol");
        let __query = __query.argument("protocol", protocol);
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field reset.\n\nSelects GraphQL field `reset` on `Greeter`."
    )]
    pub async fn reset(&self) -> ::core::result::Result<(), dagger_sdk::QueryError> {
        let __query = self.query.select("reset");
        __query.execute_void().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field shout.\n\nSelects GraphQL field `shout` on `Greeter`."
    )]
    pub async fn shout(
        &self,
        name: impl ::core::convert::Into<::std::string::String>,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("shout");
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `shout` with a borrowed, reusable `GreeterShoutOpts` value."
    )]
    pub async fn shout_opts(
        &self,
        name: impl ::core::convert::Into<::std::string::String>,
        opts: &GreeterShoutOpts,
    ) -> ::core::result::Result<::std::string::String, dagger_sdk::QueryError> {
        let __query = self.query.select("shout");
        let __query = if let ::core::option::Option::Some(__value) = &opts.mode {
            __query.argument("mode", __value)
        } else {
            __query
        };
        let __query = __query.argument(
            "name",
            ::core::convert::Into::<::std::string::String>::into(name),
        );
        let __query = if let ::core::option::Option::Some(__value) = &opts.suffixes {
            __query.argument("suffixes", __value)
        } else {
            __query
        };
        let __query = if let ::core::option::Option::Some(__value) = &opts.times {
            __query.argument("times", __value)
        } else {
            __query
        };
        __query.execute().await
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field sync.\n\nSelects GraphQL field `sync` on `Greeter`."
    )]
    pub async fn sync(&self) -> ::core::result::Result<super::Greeter, dagger_sdk::QueryError> {
        let __query = self.query.select("sync");
        let __id: dagger_sdk::Id = __query.execute().await?;
        ::core::result::Result::Ok(super::Greeter::__from_query(
            __query.generated_reentry_builder(__id, "Greeter"),
        ))
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field withMessage.\n\nSelects GraphQL field `withMessage` on `Greeter`."
    )]
    #[must_use]
    pub fn with_message(
        &self,
        message: impl ::core::convert::Into<dagger_sdk::IdInput<super::GreeterMessage>>,
    ) -> super::Greeter {
        let __query = self.query.select("withMessage");
        let __query = __query.generated_argument_id_shape(
            "message",
            ::core::convert::Into::<dagger_sdk::IdInput<super::GreeterMessage>>::into(message),
        );
        super::Greeter::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field withMessages.\n\nSelects GraphQL field `withMessages` on `Greeter`."
    )]
    #[must_use]
    pub fn with_messages(
        &self,
        messages: ::std::vec::Vec<
            ::core::option::Option<dagger_sdk::IdInput<super::GreeterMessage>>,
        >,
    ) -> super::Greeter {
        let __query = self.query.select("withMessages");
        let __query = __query.generated_argument_id_shape("messages", messages);
        super::Greeter::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field withMode.\n\nSelects GraphQL field `withMode` on `Greeter`."
    )]
    #[must_use]
    pub fn with_mode(&self, mode: super::GreeterMode) -> super::Greeter {
        let __query = self.query.select("withMode");
        let __query = __query.argument("mode", mode);
        super::Greeter::__from_query(__query)
    }
}
#[cfg_attr(
    not(doctest),
    doc = "Owned optional arguments for GraphQL operation `Query.greeter`; reuse does not mutate caller state."
)]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct QueryGreeterOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument greeting.\n\n`None` omits GraphQL field `greeting` and preserves engine default `String(\"hello\")`."
    )]
    pub greeting: ::core::option::Option<::std::string::String>,
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument mode.\n\n`None` omits GraphQL field `mode`."
    )]
    pub mode: ::core::option::Option<super::GreeterMode>,
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture argument source.\n\n`None` omits GraphQL field `source`."
    )]
    pub source: ::core::option::Option<dagger_sdk::IdInput<dagger_sdk::Directory>>,
}
impl QueryGreeterOpts {
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `greeting` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_greeting(
        mut self,
        value: impl ::core::convert::Into<::std::string::String>,
    ) -> Self {
        self.greeting = ::core::option::Option::Some(::core::convert::Into::into(value));
        self
    }
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `mode` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_mode(mut self, value: super::GreeterMode) -> Self {
        self.mode = ::core::option::Option::Some(value);
        self
    }
    #[cfg_attr(
        not(doctest),
        doc = "Sets GraphQL argument `source` to a concrete value instead of omitting it."
    )]
    #[must_use]
    pub fn with_source(mut self, value: dagger_sdk::IdInput<dagger_sdk::Directory>) -> Self {
        self.source = ::core::option::Option::Some(value);
        self
    }
}
impl super::ModuleQuery {
    #[cfg_attr(
        not(doctest),
        doc = "Client fixture field greeter.\n\nSelects GraphQL field `greeter` on `Query`."
    )]
    #[must_use]
    pub fn greeter(&self) -> super::Greeter {
        let __query = self.__query_builder().select("greeter");
        super::Greeter::__from_query(__query)
    }
    #[cfg_attr(
        not(doctest),
        doc = "Executes GraphQL operation `greeter` with a borrowed, reusable `QueryGreeterOpts` value."
    )]
    #[must_use]
    pub fn greeter_opts(&self, opts: &QueryGreeterOpts) -> super::Greeter {
        let __query = self.__query_builder().select("greeter");
        let __query = if let ::core::option::Option::Some(__value) = &opts.greeting {
            __query.argument("greeting", __value)
        } else {
            __query
        };
        let __query = if let ::core::option::Option::Some(__value) = &opts.mode {
            __query.argument("mode", __value)
        } else {
            __query
        };
        let __query = if let ::core::option::Option::Some(__value) = &opts.source {
            __query.generated_argument_id_shape("source", ::core::clone::Clone::clone(__value))
        } else {
            __query
        };
        super::Greeter::__from_query(__query)
    }
}
