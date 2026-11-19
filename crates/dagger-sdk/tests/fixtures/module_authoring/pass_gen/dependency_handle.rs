// A module's generated code defines handles and enums for the types of the modules it
// depends on, in the module's own crate. This file is written by hand: it is not
// generated, and nothing compares it with the renderer in `dagger-codegen`, whose own
// tests build and lint what it really emits. What this fixture proves, inside this
// crate's tests, is that the SDK lets another crate do what that generated code needs:
// implement the module codecs for its own handle and enum beside the SDK's blanket
// implementations for Core handles, convert its handle to the ID input of a Core
// interface, and extend a query root's loading to its handles.
#![deny(warnings)]

use dagger_sdk as renamed_sdk;

mod dagger_generated {
    pub mod __private {
        pub use crate::renamed_sdk::__private::*;
    }

    use crate::renamed_sdk;

    /// Query root bound to the active session.
    #[derive(Clone)]
    pub struct ModuleQuery {
        root: renamed_sdk::Query,
        builder: renamed_sdk::QueryBuilder,
    }

    /// A generated object or interface handle that `ModuleQuery` can load from an ID.
    pub trait ModuleLoadable: Sized {
        #[doc(hidden)]
        fn __module_ref(query: &renamed_sdk::QueryBuilder, id: renamed_sdk::IdInput<Self>) -> Self;
        #[doc(hidden)]
        fn __module_load(
            query: &renamed_sdk::QueryBuilder,
            id: renamed_sdk::IdInput<Self>,
        ) -> impl core::future::Future<Output = Result<Self, renamed_sdk::QueryError>> + Send;
    }

    impl<T> ModuleLoadable for T
    where
        T: renamed_sdk::Loadable + Send + 'static,
    {
        fn __module_ref(query: &renamed_sdk::QueryBuilder, id: renamed_sdk::IdInput<Self>) -> Self {
            query.generated_query_root().r#ref(id)
        }

        fn __module_load(
            query: &renamed_sdk::QueryBuilder,
            id: renamed_sdk::IdInput<Self>,
        ) -> impl core::future::Future<Output = Result<Self, renamed_sdk::QueryError>> + Send
        {
            let root = query.generated_query_root();
            async move { root.load(id).await }
        }
    }

    impl ModuleQuery {
        pub fn new(builder: renamed_sdk::QueryBuilder) -> Self {
            Self {
                root: builder.generated_query_root(),
                builder,
            }
        }

        pub(super) fn __query_builder(&self) -> &renamed_sdk::QueryBuilder {
            &self.builder
        }

        pub fn r#ref<T: ModuleLoadable>(&self, id: impl Into<renamed_sdk::IdInput<T>>) -> T {
            T::__module_ref(self.__query_builder(), id.into())
        }

        pub async fn load<T: ModuleLoadable>(
            &self,
            id: impl Into<renamed_sdk::IdInput<T>>,
        ) -> Result<T, renamed_sdk::QueryError> {
            T::__module_load(self.__query_builder(), id.into()).await
        }
    }

    impl core::ops::Deref for ModuleQuery {
        type Target = renamed_sdk::Query;

        fn deref(&self) -> &Self::Target {
            &self.root
        }
    }

    mod greeter {
        use crate::renamed_sdk;

        #[derive(Clone)]
        pub struct Greeter {
            query: renamed_sdk::QueryBuilder,
        }

        impl renamed_sdk::IntoID<renamed_sdk::Id> for Greeter {
            fn into_id(
                self,
            ) -> core::pin::Pin<
                Box<
                    dyn core::future::Future<
                            Output = Result<renamed_sdk::Id, renamed_sdk::QueryError>,
                        > + Send,
                >,
            > {
                Box::pin(async move { self.id().await })
            }
        }

        impl From<Greeter> for renamed_sdk::IdInput<Greeter> {
            fn from(value: Greeter) -> Self {
                renamed_sdk::IdInput::generated_lazy(value)
            }
        }

        impl From<Greeter> for renamed_sdk::IdInput<renamed_sdk::NodeClient> {
            fn from(value: Greeter) -> Self {
                renamed_sdk::IdInput::generated_lazy(value)
            }
        }

        impl renamed_sdk::__private::ModuleInputCodec for Greeter {
            fn decode_input(
                value: &renamed_sdk::__private::ModuleJson,
                context: &renamed_sdk::__private::ModuleContextBase,
            ) -> Result<Self, renamed_sdk::__private::DecodeError> {
                renamed_sdk::__private::decode_handle_input(value, context, "Greeter")
                    .map(Self::__from_query)
            }
        }

        impl renamed_sdk::__private::ModuleOutputCodec for Greeter {
            fn encode_output<'a>(
                self,
                _context: &'a renamed_sdk::__private::ModuleContextBase,
            ) -> renamed_sdk::__private::ModuleBoxFuture<
                'a,
                Result<renamed_sdk::__private::ModuleJson, renamed_sdk::__private::EncodeError>,
            >
            where
                Self: 'a,
            {
                renamed_sdk::__private::encode_handle_output(self)
            }
        }

        impl super::ModuleLoadable for Greeter {
            fn __module_ref(
                query: &renamed_sdk::QueryBuilder,
                id: renamed_sdk::IdInput<Self>,
            ) -> Self {
                Self::__from_query(query.generated_lazy_reentry_builder(id, "Greeter"))
            }

            fn __module_load(
                query: &renamed_sdk::QueryBuilder,
                id: renamed_sdk::IdInput<Self>,
            ) -> impl core::future::Future<Output = Result<Self, renamed_sdk::QueryError>> + Send
            {
                let query = query.clone();
                async move {
                    let id: renamed_sdk::Id = query
                        .generated_lazy_reentry_builder(id, "Greeter")
                        .select("id")
                        .execute()
                        .await?;
                    Ok(Self::__from_query(
                        query.generated_reentry_builder(id, "Greeter"),
                    ))
                }
            }
        }

        impl Greeter {
            #[doc(hidden)]
            pub fn __from_query(query: renamed_sdk::QueryBuilder) -> Self {
                Self { query }
            }

            pub async fn hello(
                &self,
                name: impl Into<String>,
            ) -> Result<String, renamed_sdk::QueryError> {
                let query = self.query.select("hello");
                let query = query.argument("name", name.into());
                query.execute().await
            }

            pub async fn id(&self) -> Result<renamed_sdk::Id, renamed_sdk::QueryError> {
                let query = self.query.select("id");
                query.execute().await
            }

            pub async fn mode(&self) -> Result<super::GreeterMode, renamed_sdk::QueryError> {
                let query = self.query.select("mode");
                query.execute().await
            }

            #[must_use]
            pub fn with_source(
                &self,
                source: impl Into<renamed_sdk::IdInput<renamed_sdk::Directory>>,
            ) -> Greeter {
                let query = self.query.select("withSource");
                let query = query.generated_argument_id_shape("source", source.into());
                Greeter::__from_query(query)
            }
        }

        impl super::ModuleQuery {
            #[must_use]
            pub fn greeter(&self) -> Greeter {
                let query = self.__query_builder().select("greeter");
                Greeter::__from_query(query)
            }
        }
    }
    pub use greeter::*;

    mod greeter_mode {
        use crate::renamed_sdk;

        #[derive(
            Clone,
            Copy,
            Debug,
            Eq,
            Hash,
            PartialEq,
            renamed_sdk::__private::serde::Deserialize,
            renamed_sdk::__private::serde::Serialize,
        )]
        #[serde(crate = "renamed_sdk::__private::serde")]
        pub enum GreeterMode {
            #[serde(rename = "LOUD")]
            Loud,
            #[serde(rename = "QUIET")]
            Quiet,
        }

        impl renamed_sdk::__private::ModuleInputCodec for GreeterMode {
            fn decode_input(
                value: &renamed_sdk::__private::ModuleJson,
                _context: &renamed_sdk::__private::ModuleContextBase,
            ) -> Result<Self, renamed_sdk::__private::DecodeError> {
                renamed_sdk::__private::decode_string_value(value)
            }
        }

        impl renamed_sdk::__private::ModuleOutputCodec for GreeterMode {
            fn encode_output<'a>(
                self,
                _context: &'a renamed_sdk::__private::ModuleContextBase,
            ) -> renamed_sdk::__private::ModuleBoxFuture<
                'a,
                Result<renamed_sdk::__private::ModuleJson, renamed_sdk::__private::EncodeError>,
            >
            where
                Self: 'a,
            {
                renamed_sdk::__private::encode_string_value(self)
            }
        }
    }
    pub use greeter_mode::*;
}

use dagger_generated::{Greeter, GreeterMode, ModuleQuery};

#[renamed_sdk::object(root, rename = "welcome")]
pub(crate) struct Welcome {
    #[dagger(state)]
    greeter: Greeter,
    #[dagger(state)]
    mode: GreeterMode,
}

#[renamed_sdk::methods]
impl Welcome {
    #[dagger(function)]
    async fn greet(&self, name: String) -> Result<String, renamed_sdk::ModuleError> {
        let _ = self.mode;
        Ok(self.greeter.hello(name).await?)
    }
}

async fn calls(
    query: &ModuleQuery,
    source: renamed_sdk::Directory,
) -> Result<(), renamed_sdk::QueryError> {
    let greeter = query.greeter().with_source(source);
    let _: GreeterMode = greeter.mode().await?;
    let id = greeter.id().await?;
    // One spelling loads a Core handle and a dependency's, lazily or checked.
    let _: Greeter = query.r#ref(id.clone());
    let _: Greeter = query.load(greeter.clone()).await?;
    let _: renamed_sdk::Container = query.r#ref(id.clone());
    let _: renamed_sdk::Container = query.load(id).await?;
    // An argument that takes any node accepts the dependency's handle.
    let _: renamed_sdk::IdInput<renamed_sdk::NodeClient> = greeter.into();
    Ok(())
}

fn require_codecs<T: renamed_sdk::__private::ModuleValueCodec>() {}

fn require_state<T>()
where
    T: renamed_sdk::__private::ModuleObjectBridge<PersistentState = (Greeter, GreeterMode)>,
{
}

fn main() {
    require_codecs::<Greeter>();
    require_codecs::<GreeterMode>();
    require_codecs::<renamed_sdk::Container>();
    require_state::<Welcome>();
    let _ = calls;
    let _ = ModuleQuery::new;
}
