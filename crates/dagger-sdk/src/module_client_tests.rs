//! Module loading for generated module clients, against a scripted connection.
//!
//! The cases follow `module_client_test.go` in Dagger's Go SDK: loads are shared by
//! session, address and pin; a failed load is retried; an abandoned wait does not
//! abandon the load; construction is lazy and follows the session's lifecycle; and only
//! the validation error for a missing field is given a module client's meaning.

use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use pretty_assertions::assert_eq;
use serde_json::{Map, json};
use tokio::sync::Notify;

use crate::config::ClientConfig;
use crate::connection::{EngineConnection, EngineConnectionError, EngineConnectionErrorKind};
use crate::graphql::{GraphQlError, GraphQlPathSegment, RawRequest, RawResponse, ResponseData};
use crate::{ModuleClientErrorKind, QueryBuilder, QueryError, RequestError};

const SERVE_MODULE: &str = "query{serveModule(";

type Reply = Result<RawResponse, EngineConnectionError>;
type Derivation = fn(&QueryBuilder) -> QueryBuilder;

/// How the connection answers one `serveModule` request.
enum Load {
    /// Answer at once.
    Reply(Reply),
    /// Announce the request, then answer when the test releases it.
    Held(Reply),
}

#[derive(Default)]
struct Script {
    documents: Mutex<Vec<String>>,
    loads: Mutex<VecDeque<Load>>,
    reply: Mutex<Option<Reply>>,
    load_started: Notify,
    load_release: Notify,
}

/// A connection that records every document and answers from a script.
///
/// An unscripted `serveModule` succeeds after yielding once, so requests issued
/// together overlap it. Every other request gets the scripted reply, or data that
/// satisfies each selection these tests make.
#[derive(Clone, Default)]
struct ModuleProbe(Arc<Script>);

impl ModuleProbe {
    fn script_loads(&self, loads: impl IntoIterator<Item = Load>) {
        self.0.loads.lock().expect("load script").extend(loads);
    }

    fn reply_with(&self, reply: Reply) {
        *self.0.reply.lock().expect("reply script") = Some(reply);
    }

    fn documents(&self) -> Vec<String> {
        self.0.documents.lock().expect("recorded documents").clone()
    }

    fn loads(&self) -> Vec<String> {
        self.documents()
            .into_iter()
            .filter(|document| document.starts_with(SERVE_MODULE))
            .collect()
    }

    async fn load_started(&self) {
        self.0.load_started.notified().await;
    }

    fn release_load(&self) {
        self.0.load_release.notify_one();
    }
}

#[async_trait]
impl EngineConnection for ModuleProbe {
    async fn execute(&self, request: RawRequest) -> Reply {
        self.0
            .documents
            .lock()
            .expect("recorded documents")
            .push(request.query().to_owned());
        if !request.query().starts_with(SERVE_MODULE) {
            let scripted = self.0.reply.lock().expect("reply script").clone();
            return scripted.unwrap_or_else(|| Ok(data()));
        }
        let load = self.0.loads.lock().expect("load script").pop_front();
        match load {
            Some(Load::Reply(reply)) => reply,
            Some(Load::Held(reply)) => {
                // `notify_one` keeps a permit, so neither side can miss the other.
                self.0.load_started.notify_one();
                self.0.load_release.notified().await;
                reply
            }
            None => {
                tokio::task::yield_now().await;
                Ok(served())
            }
        }
    }

    async fn close(&self) -> Result<(), EngineConnectionError> {
        Ok(())
    }

    fn abort(&self) {}
}

fn served() -> RawResponse {
    RawResponse::new(ResponseData::Value(json!({"serveModule": null})))
}

fn data() -> RawResponse {
    RawResponse::new(ResponseData::Value(json!({
        "version": "v1.0.0",
        "greeter": {"hello": "hello, Ian", "base": {"stdout": "out"}},
        "node": {"hello": "hello again"},
    })))
}

fn transport_failure() -> EngineConnectionError {
    EngineConnectionError::new(EngineConnectionErrorKind::Transport)
}

fn validation_error(message: &str) -> GraphQlError {
    GraphQlError::new(message).with_extensions(Map::from_iter([(
        "code".to_owned(),
        json!("GRAPHQL_VALIDATION_FAILED"),
    )]))
}

const MISSING_FIELD: &str = r#"Cannot query field "hi" on type "Hello"."#;

fn errors(error: GraphQlError) -> RawResponse {
    RawResponse::new(ResponseData::Null).with_errors(vec![error])
}

async fn client_with(probe: &ModuleProbe) -> crate::Client {
    let config = ClientConfig::builder()
        .connection(Box::new(probe.clone()))
        .build()
        .expect("valid explicit connection");
    crate::connect_with(config)
        .await
        .expect("explicit connection bypasses source discovery")
}

async fn hello(builder: QueryBuilder) -> Result<String, QueryError> {
    builder.select("greeter").select("hello").execute().await
}

/// Lets every spawned request run until it next waits.
async fn settle() {
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
}

fn module_error(error: &QueryError) -> (ModuleClientErrorKind, &str) {
    let QueryError::ModuleClient(module) = error else {
        panic!("expected a module client error, got {error:?}");
    };
    (module.kind(), module.address())
}

#[tokio::test]
async fn loads_are_shared_by_session_address_and_pin() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;
    let address = "github.com/example/module";

    // Two builders for one address and pin, as two generated `Client` values make.
    let first = client.generated_module_query_builder(address, "one");
    let second = client.generated_module_query_builder(address, "one");
    let requests = (0..20).map(|index| {
        hello(if index % 2 == 0 {
            first.clone()
        } else {
            second.clone()
        })
    });
    for result in futures::future::join_all(requests).await {
        assert_eq!(result.expect("request after the load"), "hello, Ian");
    }
    assert_eq!(
        probe.loads(),
        [r#"query{serveModule(address:"github.com/example/module", refPin:"one")}"#]
    );
    assert_eq!(
        probe.documents()[0],
        probe.loads()[0],
        "the load precedes every request"
    );

    hello(client.generated_module_query_builder(address, "two"))
        .await
        .expect("request on another pin");
    assert_eq!(probe.loads().len(), 2, "another pin is another load");

    let other = client_with(&probe).await;
    hello(other.generated_module_query_builder(address, "one"))
        .await
        .expect("request on another session");
    assert_eq!(probe.loads().len(), 3, "another session loads again");

    hello(first).await.expect("request on a loaded module");
    assert_eq!(probe.loads().len(), 3);
}

#[tokio::test]
async fn address_and_pin_are_encoded_as_argument_literals() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;

    hello(client.generated_module_query_builder("/a\"){evil}\n\\", "p\"in"))
        .await
        .expect("request after the load");

    assert_eq!(
        probe.loads(),
        [r#"query{serveModule(address:"/a\"){evil}\n\\", refPin:"p\"in")}"#]
    );
}

#[tokio::test]
async fn a_failed_load_fails_its_waiters_and_the_next_request_retries() {
    let probe = ModuleProbe::default();
    probe.script_loads([Load::Held(Err(transport_failure()))]);
    let client = client_with(&probe).await;
    let builder = client.generated_module_query_builder("/hello", "");

    let driver = tokio::spawn(hello(builder.clone()));
    probe.load_started().await;
    let waiter = tokio::spawn(hello(builder.clone()));
    settle().await;
    probe.release_load();

    for request in [driver, waiter] {
        let error = request
            .await
            .expect("request task")
            .expect_err("the load failed");
        assert_eq!(
            module_error(&error),
            (ModuleClientErrorKind::Load, "/hello")
        );
        assert_eq!(error.to_string(), r#"load module "/hello""#);
        let cause = error
            .source()
            .and_then(|source| source.downcast_ref::<QueryError>())
            .expect("the load's query error is the source");
        assert!(matches!(
            cause,
            QueryError::Request(RequestError::Connection(_))
        ));
    }
    assert_eq!(
        probe.documents().len(),
        1,
        "no request follows a failed load"
    );

    assert_eq!(
        hello(builder.clone()).await.expect("retried load"),
        "hello, Ian"
    );
    assert_eq!(probe.loads().len(), 2);

    client.close().await.expect("close");
    let error = hello(builder).await.expect_err("request after close");
    assert!(matches!(
        error,
        QueryError::Request(RequestError::ClientClosed)
    ));
    assert_eq!(probe.loads().len(), 2);
}

#[tokio::test]
async fn a_dropped_waiter_does_not_stop_the_load() {
    let probe = ModuleProbe::default();
    probe.script_loads([Load::Held(Ok(served()))]);
    let client = client_with(&probe).await;
    let builder = client.generated_module_query_builder("/hello", "");

    let driver = tokio::spawn(hello(builder.clone()));
    probe.load_started().await;
    let waiter = tokio::spawn(hello(builder.clone()));
    settle().await;
    waiter.abort();
    assert!(waiter.await.expect_err("dropped waiter").is_cancelled());

    probe.release_load();
    assert_eq!(
        driver.await.expect("request task").expect("held load"),
        "hello, Ian"
    );
    hello(builder).await.expect("request on a loaded module");
    assert_eq!(probe.loads().len(), 1);
}

#[tokio::test]
async fn a_waiter_takes_over_the_load_its_driver_dropped() {
    let probe = ModuleProbe::default();
    probe.script_loads([Load::Held(Ok(served()))]);
    let client = client_with(&probe).await;
    let builder = client.generated_module_query_builder("/hello", "");

    let driver = tokio::spawn(hello(builder.clone()));
    probe.load_started().await;
    let waiter = tokio::spawn(hello(builder.clone()));
    settle().await;
    driver.abort();
    assert!(driver.await.expect_err("dropped driver").is_cancelled());

    assert_eq!(
        waiter.await.expect("request task").expect("second load"),
        "hello, Ian"
    );
    assert_eq!(probe.loads().len(), 2, "the waiter sent the load again");
    hello(builder).await.expect("request on a loaded module");
    assert_eq!(probe.loads().len(), 2);
}

#[tokio::test]
async fn loads_of_different_modules_do_not_wait_for_each_other() {
    let probe = ModuleProbe::default();
    probe.script_loads([Load::Held(Ok(served()))]);
    let client = client_with(&probe).await;

    let held = tokio::spawn(hello(client.generated_module_query_builder("/held", "")));
    probe.load_started().await;

    hello(client.generated_module_query_builder("/other", ""))
        .await
        .expect("request on another module while a load is held");
    client
        .query()
        .version()
        .await
        .expect("plain request while a load is held");

    probe.release_load();
    held.await.expect("request task").expect("held load");
    assert_eq!(probe.loads().len(), 2);
}

#[tokio::test]
async fn construction_sends_nothing() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;

    let builder = client.generated_module_query_builder("/hello", "");
    let _root = builder.generated_query_root();
    let _handle: crate::Container = builder.select("greeter").generated_core_handle();
    let reentered = builder.generated_reentry_builder(crate::Id::from("greeter-id"), "Greeter");
    assert!(probe.documents().is_empty());

    // An ID the builder already holds is answered without a request, so without a load.
    let id: crate::Id = reentered.select("id").execute().await.expect("kept ID");
    assert_eq!(id, crate::Id::from("greeter-id"));
    assert!(probe.documents().is_empty());
}

#[tokio::test]
async fn a_closed_session_refuses_a_module_request_without_loading() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;
    let builder = client.generated_module_query_builder("/hello", "");
    client.close().await.expect("close");

    let error = hello(builder).await.expect_err("request after close");

    assert!(matches!(
        error,
        QueryError::Request(RequestError::ClientClosed)
    ));
    assert!(probe.documents().is_empty());
}

#[tokio::test]
async fn only_a_missing_field_validation_error_means_out_of_date() {
    let resolver_quote = validation_error(MISSING_FIELD)
        .with_path(vec![GraphQlPathSegment::Field("hello".to_owned())]);
    let cases: [(&str, Reply, bool); 7] = [
        (
            "missing binding",
            Ok(errors(validation_error(MISSING_FIELD))),
            true,
        ),
        (
            "resolver quotes validator",
            Ok(errors(resolver_quote.clone())),
            false,
        ),
        (
            "no validation code",
            Ok(errors(GraphQlError::new(MISSING_FIELD))),
            false,
        ),
        (
            "other validation",
            Ok(errors(validation_error("Unknown argument"))),
            false,
        ),
        ("transport failure", Err(transport_failure()), false),
        (
            "missing binding in an HTTP error response",
            Err(EngineConnectionError::with_http_response(
                422,
                Some(errors(validation_error(MISSING_FIELD))),
            )),
            true,
        ),
        (
            "resolver quotes validator in an HTTP error response",
            Err(EngineConnectionError::with_http_response(
                422,
                Some(errors(resolver_quote)),
            )),
            false,
        ),
    ];

    for (name, reply, out_of_date) in cases {
        let probe = ModuleProbe::default();
        probe.reply_with(reply.clone());
        let client = client_with(&probe).await;

        let error = hello(client.generated_module_query_builder("/hello", ""))
            .await
            .expect_err(name);

        let original = if out_of_date {
            assert_eq!(
                module_error(&error),
                (ModuleClientErrorKind::OutOfDate, "/hello"),
                "{name}"
            );
            assert_eq!(
                error.to_string(),
                r#"generated client for "/hello" is out of date; regenerate it"#,
                "{name}"
            );
            error
                .source()
                .and_then(|source| source.downcast_ref::<QueryError>())
                .expect("the request's query error is the source")
        } else {
            &error
        };
        // The request's own error is kept whole, wrapped or not.
        match (original, &reply) {
            (QueryError::GraphQl { response }, Ok(sent)) => assert_eq!(response, sent, "{name}"),
            (QueryError::Request(RequestError::Connection(connection)), Err(sent)) => {
                assert_eq!(connection.http_status(), sent.http_status(), "{name}");
                assert_eq!(connection.raw_response(), sent.raw_response(), "{name}");
            }
            (other, _) => panic!("{name}: unexpected error {other:?}"),
        }
        assert_eq!(probe.loads().len(), 1, "{name}");
    }
}

#[tokio::test]
async fn a_load_failure_names_the_module_or_the_missing_engine_support() {
    let cases: [(&str, Reply, ModuleClientErrorKind, &str); 4] = [
        (
            "serveModule is not in the schema",
            Ok(errors(validation_error(
                r#"Cannot query field "serveModule" on type "Query"."#,
            ))),
            ModuleClientErrorKind::Unsupported,
            "the engine does not support generated module clients (serveModule is required)",
        ),
        (
            "serveModule is not in the schema, in an HTTP error response",
            Err(EngineConnectionError::with_http_response(
                422,
                Some(errors(validation_error(
                    r#"Cannot query field "serveModule" on type "Query"."#,
                ))),
            )),
            ModuleClientErrorKind::Unsupported,
            "the engine does not support generated module clients (serveModule is required)",
        ),
        (
            "the module does not load",
            Ok(errors(GraphQlError::new("module not found").with_path(
                vec![GraphQlPathSegment::Field("serveModule".to_owned())],
            ))),
            ModuleClientErrorKind::Load,
            r#"load module "/he\"llo""#,
        ),
        (
            "the module's own schema quotes the validator",
            Ok(errors(validation_error(MISSING_FIELD).with_path(vec![
                GraphQlPathSegment::Field("serveModule".to_owned()),
            ]))),
            ModuleClientErrorKind::Load,
            r#"load module "/he\"llo""#,
        ),
    ];

    for (name, reply, kind, message) in cases {
        let probe = ModuleProbe::default();
        probe.script_loads([Load::Reply(reply.clone())]);
        let client = client_with(&probe).await;

        let error = hello(client.generated_module_query_builder("/he\"llo", ""))
            .await
            .expect_err(name);

        assert_eq!(module_error(&error), (kind, "/he\"llo"), "{name}");
        assert_eq!(error.to_string(), message, "{name}");
        let cause = error
            .source()
            .and_then(|source| source.downcast_ref::<QueryError>())
            .expect("the load's query error is the source");
        match (cause, &reply) {
            (QueryError::GraphQl { response }, Ok(sent)) => assert_eq!(response, sent, "{name}"),
            (QueryError::Request(RequestError::Connection(connection)), Err(sent)) => {
                assert_eq!(connection.raw_response(), sent.raw_response(), "{name}");
            }
            (other, _) => panic!("{name}: unexpected cause {other:?}"),
        }
        assert_eq!(probe.documents().len(), 1, "{name}: only the load was sent");
    }
}

#[tokio::test]
async fn handles_derived_from_a_module_builder_load_the_module_first() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;
    let builder = client.generated_module_query_builder("/hello", "");

    let version = builder
        .generated_query_root()
        .version()
        .await
        .expect("Core field on the module builder's root");
    assert_eq!(version, "v1.0.0");
    assert_eq!(
        probe.documents(),
        [
            r#"query{serveModule(address:"/hello", refPin:"")}"#,
            "query{version}"
        ]
    );

    // Each derivation is tried on a session of its own, so each must load.
    let derivations: [(&str, Derivation); 4] = [
        ("select", |builder| builder.select("greeter")),
        ("argument", |builder| {
            builder.select("greeter").argument("greeting", "hi")
        }),
        ("re-entry", |builder| {
            builder.generated_reentry_builder(crate::Id::from("greeter-id"), "Greeter")
        }),
        ("lazy re-entry", |builder| {
            builder.generated_lazy_reentry_builder(
                crate::IdInput::<crate::Id>::new(crate::Id::from("greeter-id")),
                "Greeter",
            )
        }),
    ];
    for (name, derive) in derivations {
        let probe = ModuleProbe::default();
        let client = client_with(&probe).await;
        let derived = derive(&client.generated_module_query_builder("/hello", ""));

        let _: String = derived.select("hello").execute().await.expect(name);

        assert_eq!(probe.loads().len(), 1, "{name}");
        assert!(probe.documents()[0].starts_with(SERVE_MODULE), "{name}");
    }

    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;
    let container: crate::Container = client
        .generated_module_query_builder("/hello", "")
        .select("greeter")
        .select("base")
        .generated_core_handle();
    assert_eq!(container.stdout().await.expect("Core handle"), "out");
    assert_eq!(
        probe.documents(),
        [
            r#"query{serveModule(address:"/hello", refPin:"")}"#,
            "query{greeter{base{stdout}}}"
        ]
    );
}

#[tokio::test]
async fn a_plain_query_never_loads_a_module() {
    let probe = ModuleProbe::default();
    let client = client_with(&probe).await;
    let _unused = client.generated_module_query_builder("/hello", "");

    client.query().version().await.expect("plain root query");
    let _: String = client
        .query_builder()
        .select("version")
        .execute()
        .await
        .expect("plain builder");

    assert_eq!(probe.documents(), ["query{version}", "query{version}"]);

    // A missing field means nothing about a module when no module is required.
    probe.reply_with(Ok(errors(validation_error(MISSING_FIELD))));
    let error = client
        .query()
        .version()
        .await
        .expect_err("validation failure");
    assert!(matches!(error, QueryError::GraphQl { .. }));
    assert!(probe.loads().is_empty());
}
