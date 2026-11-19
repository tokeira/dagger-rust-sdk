//! Version-locked typed bridges between authored items and generated module code.
//!
//! Procedural macros implement these traits inside the authored crate. Generated code
//! re-exports them through `crate::dagger_generated::__private`, so expansion never
//! depends on the spelling of the SDK dependency in `Cargo.toml`.

/// Hidden bridge ABI consumed by generated module support.
#[doc(hidden)]
pub mod __private {
    use std::error::Error;
    use std::fmt;
    use std::future::Future;
    use std::pin::Pin;

    use super::super::context::ModuleContextBase;
    use super::super::wire::ModuleJson;

    /// Current syntactic contract shared by source analysis and procedural macros.
    pub const AUTHORING_ABI_VERSION: u32 = 1;

    /// Boxed generated future whose lifetime remains tied to the active call.
    pub type ModuleBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// Typed failure to decode one module boundary value.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DecodeError;

    impl fmt::Display for DecodeError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("module input decoding failed")
        }
    }

    impl Error for DecodeError {}

    /// Typed failure to encode one module boundary value.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct EncodeError;

    impl fmt::Display for EncodeError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("module output encoding failed")
        }
    }

    impl Error for EncodeError {}

    /// Const-generic witness for one normalized authoring declaration.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct AuthoringFingerprint<const VALUE: u128>;

    /// Owned persistent-state access emitted for an authored object.
    pub trait ModuleObjectBridge: Sized {
        /// Tuple of exposed and private-persistent fields in authored order.
        type PersistentState;
        /// Const-generic witness of the normalized object declaration.
        type Fingerprint;

        /// Reconstructs the object without bypassing normal Rust initialization.
        fn from_persistent_state(state: Self::PersistentState) -> Self;

        /// Extracts only fields explicitly declared persistent.
        fn into_persistent_state(self) -> Self::PersistentState;

        /// Returns the declaration fingerprint expected by generated code.
        fn authoring_fingerprint() -> Self::Fingerprint;
    }

    /// Fingerprint witness emitted for an authored interface.
    pub trait ModuleInterfaceBridge {
        /// Const-generic witness of the normalized interface declaration.
        type Fingerprint;

        /// Returns the declaration fingerprint expected by generated code.
        fn authoring_fingerprint() -> Self::Fingerprint;
    }

    /// Fingerprint witness emitted for an authored enum.
    pub trait ModuleEnumBridge {
        /// Const-generic witness of the normalized enum declaration.
        type Fingerprint;

        /// Returns the declaration fingerprint expected by generated code.
        fn authoring_fingerprint() -> Self::Fingerprint;
    }

    /// Lossless owned access to an authored transparent scalar newtype.
    pub trait ModuleScalarBridge: Sized {
        /// Transparent scalar representation checked by generated codecs.
        type Representation;
        /// Const-generic witness of the normalized scalar declaration.
        type Fingerprint;

        /// Wraps one representation without transformation.
        fn from_representation(value: Self::Representation) -> Self;

        /// Unwraps one representation without transformation.
        fn into_representation(self) -> Self::Representation;

        /// Returns the declaration fingerprint expected by generated code.
        fn authoring_fingerprint() -> Self::Fingerprint;
    }

    /// Per-function fingerprint witness emitted by the `methods` attribute.
    pub trait ModuleMethodBridge<const VALUE: u128> {
        /// Returns the function declaration fingerprint expected by generated code.
        fn authoring_fingerprint() -> AuthoringFingerprint<VALUE>;
    }

    /// Generated input conversion for one exact descriptor type.
    pub trait ModuleInputCodec: Sized {
        /// Decodes one canonical input using the active session when handles re-enter.
        fn decode_input(
            value: &ModuleJson,
            context: &ModuleContextBase,
        ) -> Result<Self, DecodeError>;
    }

    /// Generated output conversion for one exact descriptor type.
    pub trait ModuleOutputCodec: Sized {
        /// Encodes one value, resolving generated handle IDs on the active session.
        fn encode_output<'a>(
            self,
            context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a;
    }

    /// Value with both exact input and output conversion.
    pub trait ModuleValueCodec: ModuleInputCodec + ModuleOutputCodec {}

    impl<T> ModuleValueCodec for T where T: ModuleInputCodec + ModuleOutputCodec {}

    impl ModuleInputCodec for String {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value
                .as_json()
                .as_str()
                .map(str::to_owned)
                .ok_or(DecodeError)
        }
    }

    impl ModuleOutputCodec for String {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move { Ok(ModuleJson::new(serde_json::Value::String(self))) })
        }
    }

    impl ModuleInputCodec for i64 {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value.as_json().as_i64().ok_or(DecodeError)
        }
    }

    impl ModuleOutputCodec for i64 {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move { Ok(ModuleJson::new(serde_json::Value::Number(self.into()))) })
        }
    }

    impl ModuleInputCodec for bool {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value.as_json().as_bool().ok_or(DecodeError)
        }
    }

    impl ModuleOutputCodec for bool {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move { Ok(ModuleJson::new(serde_json::Value::Bool(self))) })
        }
    }

    impl ModuleInputCodec for f64 {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value.as_json().as_f64().ok_or(DecodeError)
        }
    }

    impl ModuleOutputCodec for f64 {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move {
                serde_json::Number::from_f64(self)
                    .map(serde_json::Value::Number)
                    .map(ModuleJson::new)
                    .ok_or(EncodeError)
            })
        }
    }

    impl ModuleInputCodec for () {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value.as_json().is_null().then_some(()).ok_or(DecodeError)
        }
    }

    impl ModuleOutputCodec for () {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move { Ok(ModuleJson::new(serde_json::Value::Null)) })
        }
    }

    impl<T> ModuleInputCodec for Vec<T>
    where
        T: ModuleInputCodec,
    {
        fn decode_input(
            value: &ModuleJson,
            context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            value
                .as_json()
                .as_array()
                .ok_or(DecodeError)?
                .iter()
                .map(|value| T::decode_input(&ModuleJson::new(value.clone()), context))
                .collect()
        }
    }

    impl<T> ModuleOutputCodec for Vec<T>
    where
        T: ModuleOutputCodec + Send,
    {
        fn encode_output<'a>(
            self,
            context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move {
                let mut encoded = Vec::with_capacity(self.len());
                for value in self {
                    encoded.push(value.encode_output(context).await?.into_json());
                }
                Ok(ModuleJson::new(serde_json::Value::Array(encoded)))
            })
        }
    }

    impl<T> ModuleInputCodec for Option<T>
    where
        T: ModuleInputCodec,
    {
        fn decode_input(
            value: &ModuleJson,
            context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            if value.as_json().is_null() {
                Ok(None)
            } else {
                T::decode_input(value, context).map(Some)
            }
        }
    }

    impl<T> ModuleOutputCodec for Option<T>
    where
        T: ModuleOutputCodec + Send,
    {
        fn encode_output<'a>(
            self,
            context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            Box::pin(async move {
                match self {
                    Some(value) => value.encode_output(context).await,
                    None => Ok(ModuleJson::new(serde_json::Value::Null)),
                }
            })
        }
    }

    /// Decodes a Core enum or scalar, which crosses the boundary as one JSON string.
    ///
    /// Generated Core enums and scalars implement [`ModuleInputCodec`] through this
    /// function, so an enum accepts its canonical members and their aliases.
    pub fn decode_string_value<T>(value: &ModuleJson) -> Result<T, DecodeError>
    where
        T: serde::de::DeserializeOwned,
    {
        if !value.as_json().is_string() {
            return Err(DecodeError);
        }
        T::deserialize(value.as_json()).map_err(|_| DecodeError)
    }

    /// Encodes a Core enum or scalar as one JSON string, an enum as its canonical member.
    pub fn encode_string_value<'a, T>(
        value: T,
    ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
    where
        T: serde::Serialize + Send + 'a,
    {
        Box::pin(async move {
            match serde_json::to_value(value) {
                Ok(value @ serde_json::Value::String(_)) => Ok(ModuleJson::new(value)),
                _ => Err(EncodeError),
            }
        })
    }

    impl ModuleInputCodec for crate::Json {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            decode_string_value(value)
        }
    }

    impl ModuleOutputCodec for crate::Json {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            encode_string_value(self)
        }
    }

    impl ModuleInputCodec for crate::Platform {
        fn decode_input(
            value: &ModuleJson,
            _context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            decode_string_value(value)
        }
    }

    impl ModuleOutputCodec for crate::Platform {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            encode_string_value(self)
        }
    }

    /// Reads the ID and concrete GraphQL type of a handle that crosses the boundary.
    ///
    /// A handle arrives as its ID, or as an object of `id` and `__typename` when the
    /// engine states the concrete type behind an interface. Anything else is refused,
    /// and so is an ID or type name that could not be one the engine wrote.
    fn handle_identity<'a>(
        value: &'a ModuleJson,
        graphql_type: &'a str,
    ) -> Result<(&'a str, &'a str), DecodeError> {
        let (id, concrete_type) = match value.as_json() {
            serde_json::Value::String(id) => (id.as_str(), graphql_type),
            serde_json::Value::Object(object)
                if object.keys().all(|key| key == "id" || key == "__typename") =>
            {
                let id = object
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or(DecodeError)?;
                let concrete_type = object
                    .get("__typename")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(graphql_type);
                (id, concrete_type)
            }
            _ => return Err(DecodeError),
        };
        if id.is_empty()
            || id.len() > 16 * 1024
            || id.contains('\0')
            || concrete_type.is_empty()
            || !concrete_type
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            return Err(DecodeError);
        }
        Ok((id, concrete_type))
    }

    /// Decodes a handle that a module's generated bindings define, such as an object of
    /// a module it depends on, into the query that loads it on the active session.
    ///
    /// `graphql_type` is the handle's own GraphQL type, used unless the value states a
    /// more specific one. The value is validated exactly as the codec of a Core handle
    /// validates it, and no request is sent. The returned builder does not keep the ID,
    /// as `QueryBuilder::reenter_generated_handle` does not: the engine passes arguments
    /// and state as IDs that need not be in the form the `id` field returns.
    pub fn decode_handle_input(
        value: &ModuleJson,
        context: &ModuleContextBase,
        graphql_type: &str,
    ) -> Result<crate::QueryBuilder, DecodeError> {
        let (id, concrete_type) = handle_identity(value, graphql_type)?;
        Ok(context.query_builder().generated_lazy_reentry_builder(
            crate::IdInput::<crate::Id>::new(crate::Id::new(id)),
            concrete_type,
        ))
    }

    /// Encodes a handle as its ID, which it resolves once on its own session.
    ///
    /// The codec of a Core handle and the codecs a module's generated bindings define
    /// for their own handles share this function, so both refuse the same IDs.
    pub fn encode_handle_output<'a, T>(
        handle: T,
    ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
    where
        T: crate::IntoID<crate::Id>,
    {
        Box::pin(async move {
            let id = handle.into_id().await.map_err(|_| EncodeError)?;
            if id.as_str().is_empty() || id.as_str().contains('\0') {
                return Err(EncodeError);
            }
            Ok(ModuleJson::new(serde_json::Value::String(id.into_inner())))
        })
    }

    #[cfg(feature = "gen")]
    impl<T> ModuleInputCodec for T
    where
        T: crate::Loadable + 'static,
    {
        fn decode_input(
            value: &ModuleJson,
            context: &ModuleContextBase,
        ) -> Result<Self, DecodeError> {
            let (id, concrete_type) = handle_identity(
                value,
                <T as crate::loadable::private::Sealed>::graphql_type(),
            )?;
            Ok(context
                .query_builder()
                .reenter_generated_handle(crate::Id::new(id), concrete_type))
        }
    }

    #[cfg(feature = "gen")]
    impl<T> ModuleOutputCodec for T
    where
        T: crate::Loadable + crate::IntoID<crate::Id> + Send + 'static,
    {
        fn encode_output<'a>(
            self,
            _context: &'a ModuleContextBase,
        ) -> ModuleBoxFuture<'a, Result<ModuleJson, EncodeError>>
        where
            Self: 'a,
        {
            encode_handle_output(self)
        }
    }
}

#[cfg(all(test, feature = "gen"))]
mod tests {
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use proptest::prelude::*;
    use serde_json::json;

    use super::__private::{
        ModuleInputCodec, ModuleOutputCodec, decode_handle_input, encode_handle_output,
    };
    use crate::connection::{EngineConnection, EngineConnectionError, EngineConnectionErrorKind};
    use crate::graphql::{RawRequest, RawResponse, ResponseData};
    use crate::lifecycle::SessionHandle;
    use crate::module::context::{CurrentCall, ModuleCancellation, ModuleContextBase};
    use crate::module::wire::{CallSelector, ModuleFunctionName, ModuleJson, ModuleWireName};
    use crate::query::QueryBuilder;
    use crate::test_support::proptest_config;
    use crate::{Id, IdInput, IntoID, QueryError};

    #[derive(Clone)]
    struct RecordingConnection {
        requests: Arc<AtomicUsize>,
        documents: Arc<Mutex<Vec<String>>>,
        id: String,
        fail: bool,
    }

    #[async_trait]
    impl EngineConnection for RecordingConnection {
        async fn execute(&self, request: RawRequest) -> Result<RawResponse, EngineConnectionError> {
            self.requests.fetch_add(1, Ordering::SeqCst);
            self.documents
                .lock()
                .expect("document lock")
                .push(request.query().to_owned());
            if self.fail {
                Err(EngineConnectionError::new(
                    EngineConnectionErrorKind::Unavailable,
                ))
            } else {
                Ok(RawResponse::new(ResponseData::Value(json!({
                    "node": {"id": self.id}
                }))))
            }
        }

        async fn close(&self) -> Result<(), EngineConnectionError> {
            Ok(())
        }

        fn abort(&self) {}
    }

    fn context(id: &str, fail: bool) -> (ModuleContextBase, Arc<AtomicUsize>) {
        let (context, requests, _) = recording_context(id, fail);
        (context, requests)
    }

    fn recording_context(
        id: &str,
        fail: bool,
    ) -> (ModuleContextBase, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
        let requests = Arc::new(AtomicUsize::new(0));
        let documents = Arc::new(Mutex::new(Vec::new()));
        let connection = RecordingConnection {
            requests: Arc::clone(&requests),
            documents: Arc::clone(&documents),
            id: id.to_owned(),
            fail,
        };
        let builder = QueryBuilder::new(SessionHandle::new(Box::new(connection), None, None));
        let selector = CallSelector::Invocation {
            parent_wire_name: ModuleWireName::new("Fixture").expect("static name is valid"),
            function_wire_name: ModuleFunctionName::new("run").expect("static name is valid"),
        };
        (
            ModuleContextBase::new(
                builder,
                ModuleCancellation::default(),
                opentelemetry::Context::new(),
                CurrentCall::new("call", selector).expect("static identity is valid"),
            ),
            requests,
            documents,
        )
    }

    /// A handle as a module's generated bindings define one for a type of a module it
    /// depends on: a query builder, and an ID read through the `id` field.
    #[derive(Clone)]
    struct DependencyHandle(QueryBuilder);

    impl IntoID<Id> for DependencyHandle {
        fn into_id(
            self,
        ) -> Pin<Box<dyn core::future::Future<Output = Result<Id, QueryError>> + Send>> {
            Box::pin(async move { self.0.select("id").execute().await })
        }
    }

    /// Every JSON shape a handle codec must judge: the two accepted ones, and each
    /// way an ID, a type name or the value's shape can be wrong.
    fn handle_value() -> impl Strategy<Value = serde_json::Value> {
        let id = prop_oneof![
            "[a-zA-Z0-9_=-]{1,64}".prop_map(serde_json::Value::String),
            Just(json!("")),
            Just(json!("with\u{0}nul")),
            Just(json!("x".repeat(16 * 1024 + 1))),
            Just(json!(7)),
            Just(serde_json::Value::Null),
        ];
        let type_name = prop_oneof![
            Just(json!("Container")),
            Just(json!("Greeter_2")),
            Just(json!("")),
            Just(json!("Greeter{id}")),
            Just(json!(7)),
        ];
        prop_oneof![
            id.clone(),
            id.clone().prop_map(|id| json!({"id": id})),
            (id.clone(), type_name.clone())
                .prop_map(|(id, type_name)| json!({"id": id, "__typename": type_name})),
            type_name.prop_map(|type_name| json!({"__typename": type_name})),
            id.clone().prop_map(|id| json!({"id": id, "extra": true})),
            id.prop_map(|id| json!([id])),
        ]
    }

    #[test]
    fn core_enums_and_scalars_cross_the_boundary_as_one_string() {
        let (context, requests) = context("unused", false);
        let decode = |value: serde_json::Value| ModuleJson::new(value);

        // An alias decodes to its member, which encodes as the canonical value.
        let file_type = <crate::FileType as ModuleInputCodec>::decode_input(
            &decode(json!("REGULAR_TYPE")),
            &context,
        )
        .expect("an alias decodes");
        assert_eq!(file_type, crate::FileType::Regular);
        let encoded = futures::executor::block_on(file_type.encode_output(&context))
            .expect("a member encodes");
        assert_eq!(encoded.as_json(), &json!("REGULAR"));
        for value in [json!("SCTP"), json!(1), json!(null)] {
            assert!(
                <crate::NetworkProtocol as ModuleInputCodec>::decode_input(
                    &decode(value),
                    &context
                )
                .is_err()
            );
        }

        // A scalar keeps its exact text; a `JSON` value is the text of a JSON document.
        let labels =
            <crate::Json as ModuleInputCodec>::decode_input(&decode(json!("{\"a\":1}")), &context)
                .expect("JSON text decodes");
        assert_eq!(labels.as_str(), "{\"a\":1}");
        let encoded =
            futures::executor::block_on(labels.encode_output(&context)).expect("JSON text encodes");
        assert_eq!(encoded.as_json(), &json!("{\"a\":1}"));
        assert!(
            <crate::Json as ModuleInputCodec>::decode_input(&decode(json!({"a": 1})), &context)
                .is_err()
        );
        let platform = <crate::Platform as ModuleInputCodec>::decode_input(
            &decode(json!("linux/arm64")),
            &context,
        )
        .expect("a platform decodes");
        let encoded = futures::executor::block_on(platform.encode_output(&context))
            .expect("a platform encodes");
        assert_eq!(encoded.as_json(), &json!("linux/arm64"));
        let bytes =
            <crate::Bytes as ModuleInputCodec>::decode_input(&decode(json!("aGk=")), &context)
                .expect("base64 text decodes");
        let encoded = futures::executor::block_on(bytes.encode_output(&context))
            .expect("base64 text encodes");
        assert_eq!(encoded.as_json(), &json!("aGk="));
        assert_eq!(requests.load(Ordering::SeqCst), 0);
    }

    proptest! {
        #![proptest_config(proptest_config())]

        #[test]
        fn property_16_handle_reconstruction_retains_identity_session(
            id in "[a-zA-Z0-9_=-]{1,64}",
            interface in any::<bool>(),
            invalid in any::<bool>(),
        ) {
            let (context, requests) = context(&id, false);
            let session_identity = context.query_builder().session_identity();
            let value = if invalid {
                ModuleJson::new(serde_json::Value::String(String::new()))
            } else if interface {
                ModuleJson::new(json!({"id": id, "__typename": "Container"}))
            } else {
                ModuleJson::new(serde_json::Value::String(id.clone()))
            };

            if interface {
                let decoded = <crate::ExportableClient as ModuleInputCodec>::decode_input(&value, &context);
                if invalid {
                    prop_assert!(decoded.is_err());
                } else {
                    let handle = decoded.expect("valid interface identifier re-enters");
                    prop_assert_eq!(handle.session.identity(), session_identity);
                    let document = futures::executor::block_on(handle.selection.build())
                        .expect("re-entry selection is valid");
                    prop_assert!(document.contains("... on Container"));
                }
            } else {
                let decoded = <crate::Container as ModuleInputCodec>::decode_input(&value, &context);
                if invalid {
                    prop_assert!(decoded.is_err());
                } else {
                    let handle = decoded.expect("valid object identifier re-enters");
                    prop_assert_eq!(handle.session.identity(), session_identity);
                    let document = futures::executor::block_on(handle.selection.build())
                        .expect("re-entry selection is valid");
                    prop_assert!(document.contains("... on Container"));
                }
            }
            prop_assert_eq!(requests.load(Ordering::SeqCst), 0);
        }

        #[test]
        fn property_17_successful_values_encode_exactly_once(
            values in proptest::collection::vec(any::<i64>(), 0..12),
            optional in proptest::option::of("[a-zA-Z0-9 ]{0,32}"),
            id in "[a-zA-Z0-9_=-]{1,64}",
            fail_handle in any::<bool>(),
        ) {
            let (primitive_context, _) = context(&id, false);
            let integers = futures::executor::block_on(values.clone().encode_output(&primitive_context))
                .expect("integer lists encode");
            prop_assert_eq!(integers.as_json(), &json!(values));
            let optional_value = futures::executor::block_on(optional.clone().encode_output(&primitive_context))
                .expect("optional strings encode");
            prop_assert_eq!(optional_value.as_json(), &json!(optional));
            let unit = futures::executor::block_on(().encode_output(&primitive_context))
                .expect("unit encodes");
            prop_assert_eq!(unit.as_json(), &serde_json::Value::Null);

            let (handle_context, requests) = context(&id, fail_handle);
            let handle = <crate::Container as ModuleInputCodec>::decode_input(
                &ModuleJson::new(serde_json::Value::String(id.clone())),
                &handle_context,
            ).expect("valid handle input");
            let encoded = futures::executor::block_on(handle.encode_output(&handle_context));
            if fail_handle {
                prop_assert!(encoded.is_err());
            } else {
                let encoded = encoded.expect("handle ID resolves");
                prop_assert_eq!(encoded.as_json(), &json!(id));
            }
            prop_assert_eq!(requests.load(Ordering::SeqCst), 1);
        }

        // Invariant: a handle the generated bindings define decodes on the call's own
        // session without a request, to the type the value states, and is refused
        // exactly when a Core handle's codec refuses the same value.
        #[test]
        fn property_18_generated_handle_decoding_matches_the_core_codec(
            value in handle_value(),
        ) {
            let (context, requests) = context("unused", false);
            let session_identity = context.query_builder().session_identity();
            let value = ModuleJson::new(value);
            let core = <crate::Container as ModuleInputCodec>::decode_input(&value, &context);
            let generated = decode_handle_input(&value, &context, "Container");
            prop_assert_eq!(generated.is_ok(), core.is_ok());
            if let (Ok(builder), Ok(core)) = (generated, core) {
                prop_assert_eq!(builder.session_identity(), session_identity);
                let document = futures::executor::block_on(builder.document())
                    .expect("re-entry selection is valid");
                let core_document = futures::executor::block_on(core.selection.build())
                    .expect("re-entry selection is valid");
                prop_assert_eq!(&document, &core_document);
                let stated = value
                    .as_json()
                    .get("__typename")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Container");
                let fragment = format!("... on {stated}");
                prop_assert!(document.contains(&fragment));
            }
            prop_assert_eq!(requests.load(Ordering::SeqCst), 0);
        }

        // Invariant: encoding a handle resolves its ID with one request, and a failed
        // resolution is an encoding failure after that one request.
        #[test]
        fn property_19_generated_handle_encodes_its_id_exactly_once(
            id in "[a-zA-Z0-9_=-]{1,64}",
            fail in any::<bool>(),
        ) {
            let (context, requests, documents) = recording_context(&id, fail);
            let builder = decode_handle_input(
                &ModuleJson::new(serde_json::Value::String("argument".to_owned())),
                &context,
                "Greeter",
            )
            .expect("valid handle input");
            let encoded =
                futures::executor::block_on(encode_handle_output(DependencyHandle(builder)));
            if fail {
                prop_assert!(encoded.is_err());
            } else {
                let encoded = encoded.expect("handle ID resolves");
                prop_assert_eq!(encoded.as_json(), &json!(id));
            }
            prop_assert_eq!(requests.load(Ordering::SeqCst), 1);
            let documents = documents.lock().expect("document lock").clone();
            prop_assert_eq!(
                documents,
                vec!["query{node(id:\"argument\"){... on Greeter{id}}}".to_owned()]
            );
        }
    }

    // An empty ID and an ID with a NUL are not IDs the engine writes. Each is refused
    // after the one request that resolved it.
    #[test]
    fn an_id_the_engine_could_not_have_written_is_not_encoded() {
        for id in ["", "with\u{0}nul"] {
            let (context, requests) = context(id, false);
            let builder = context
                .query_builder()
                .generated_lazy_reentry_builder(IdInput::<Id>::new(Id::new("argument")), "Greeter");
            let encoded =
                futures::executor::block_on(encode_handle_output(DependencyHandle(builder)));
            assert!(encoded.is_err(), "{id:?}");
            assert_eq!(requests.load(Ordering::SeqCst), 1);
        }
    }

    // The engine passes arguments and state as IDs that need not be in the form `id`
    // returns, so only a builder made from an ID a query returned may answer `id`
    // itself.
    #[test]
    fn a_lazy_reentry_asks_the_engine_for_its_id_and_a_keeping_one_does_not() {
        let (context, requests, documents) = recording_context("returned", false);
        let builder = context.query_builder();

        let lazy = builder
            .generated_lazy_reentry_builder(IdInput::<Id>::new(Id::new("passed")), "Greeter");
        let id: Id = futures::executor::block_on(lazy.select("id").execute())
            .expect("the engine returns the ID");
        assert_eq!(id.as_str(), "returned");
        assert_eq!(requests.load(Ordering::SeqCst), 1);
        assert_eq!(
            documents.lock().expect("document lock").as_slice(),
            ["query{node(id:\"passed\"){... on Greeter{id}}}".to_owned()]
        );

        let keeping = builder.generated_reentry_builder(Id::new("returned"), "Greeter");
        let id: Id = futures::executor::block_on(keeping.select("id").execute())
            .expect("the kept ID is returned");
        assert_eq!(id.as_str(), "returned");
        assert_eq!(requests.load(Ordering::SeqCst), 1);

        // A handle converted to an ID input is resolved when the request is built.
        let nested = builder.generated_lazy_reentry_builder(
            IdInput::<Id>::generated_lazy(DependencyHandle(lazy)),
            "Greeter",
        );
        let document = futures::executor::block_on(nested.select("hello").document())
            .expect("the handle's ID resolves");
        assert_eq!(
            document,
            "query{node(id:\"returned\"){... on Greeter{hello}}}"
        );
        assert_eq!(requests.load(Ordering::SeqCst), 2);
    }
}
