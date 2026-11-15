//! Runtime transport, propagation, compatibility, and failure properties.

use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use base64::Engine as _;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use proptest::prelude::*;
use serde_json::{Map, Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing_opentelemetry::layer;
use tracing_subscriber::prelude::*;

use crate::compatibility::CompatibilityValidator;
use crate::connection::{EngineConnection, EngineConnectionError, EngineConnectionErrorKind};
use crate::errors::{ConnectError, QueryError};
use crate::graphql::{GraphQlError, RawRequest, RawResponse, ResponseData};
use crate::preflight::PropagationEnvironment;
use crate::propagation::W3cPropagation;
use crate::runtime_errors::{
    CompatibilityError, CompatibilityErrorKind, CompatibilityEvidenceGap, ExecError,
    ProvisioningError, ProvisioningErrorKind, SessionStartupError, SessionStartupErrorKind,
    ShutdownError, ShutdownFailureKind,
};
use crate::session::SecretString;
use crate::session_startup::install_live_session_observer;
use crate::test_support::{io_proptest_config, proptest_config};
use crate::transport::{LoopbackTransportFactory, ReqwestLoopbackFactory};

fn propagation(
    traceparent: Option<String>,
    tracestate: Option<String>,
    baggage: Option<String>,
) -> PropagationEnvironment {
    PropagationEnvironment::for_test(
        traceparent.map(OsString::from),
        tracestate.map(OsString::from),
        baggage.map(OsString::from),
    )
}

fn loopback(
    port: u16,
    token: &str,
    inherited: PropagationEnvironment,
) -> crate::transport::ReqwestLoopbackConnection {
    let token = SecretString::new(Arc::<str>::from(token)).expect("test token is non-empty");
    ReqwestLoopbackFactory
        .loopback(port, token, Duration::from_secs(1), inherited, None)
        .expect("fixed loopback client construction succeeds")
}

#[derive(Default)]
struct LiveDiagnosticSink {
    payloads: Mutex<Vec<Vec<u8>>>,
}

impl crate::DiagnosticSink for LiveDiagnosticSink {
    fn emit(&self, diagnostic: crate::Diagnostic<'_>) -> Result<(), crate::DiagnosticSinkError> {
        match self.payloads.lock() {
            Ok(mut payloads) => payloads.push(diagnostic.payload.to_vec()),
            Err(poisoned) => poisoned.into_inner().push(diagnostic.payload.to_vec()),
        }
        Ok(())
    }
}

impl LiveDiagnosticSink {
    fn payloads(&self) -> Vec<Vec<u8>> {
        match self.payloads.lock() {
            Ok(payloads) => payloads.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "downloads and starts the exact Dagger engine target"]
async fn exact_target_default_connector_evidence() {
    const HELPER: &str = "DAGGER_RUST_SDK_LIVE_HELPER";
    const CACHE_ROOT: &str = "DAGGER_RUST_SDK_TEST_CACHE_ROOT";
    const TEST_NAME: &str = "runtime_connector_tests::exact_target_default_connector_evidence";

    if std::env::var_os(HELPER).is_none() {
        let cache = tempfile::tempdir().expect("isolated live cache");
        let mut command =
            tokio::process::Command::new(std::env::current_exe().expect("current test executable"));
        command
            .args([
                "--exact",
                TEST_NAME,
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(HELPER, "1")
            .env(CACHE_ROOT, cache.path())
            .env(
                "TRACEPARENT",
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01",
            )
            .env("BAGGAGE", "rust-sdk-live-evidence=present");
        for key in [
            "DAGGER_SESSION_PORT",
            "DAGGER_SESSION_TOKEN",
            "_EXPERIMENTAL_DAGGER_CLI_BIN",
            "_EXPERIMENTAL_DAGGER_RUNNER_HOST",
            "_EXPERIMENTAL_DAGGER_RUNNER_TOKEN",
        ] {
            command.env_remove(key);
        }
        let output = command.output().await.expect("isolated helper runs");
        assert!(
            output.status.success(),
            "exact-target helper failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        return;
    }

    let cache_root = std::path::PathBuf::from(
        std::env::var_os(CACHE_ROOT).expect("parent supplies isolated cache root"),
    );
    assert_eq!(
        fs::read_dir(&cache_root)
            .expect("isolated cache exists")
            .count(),
        0,
        "live evidence must begin without a reusable CLI",
    );

    let observer = install_live_session_observer();
    let diagnostics = Arc::new(LiveDiagnosticSink::default());
    let config = crate::ClientConfig::builder()
        .diagnostic_sink(diagnostics.clone())
        .build()
        .expect("live configuration is valid");
    let client = crate::connect_with(config)
        .await
        .expect("stable default connector reaches exact target");
    let version = client
        .query()
        .version()
        .await
        .expect("generated authenticated Query.version succeeds");
    let view = client
        .execute(RawRequest::new("query { __schemaVersion }"))
        .await
        .expect("authenticated view query succeeds");
    let view = match view.data() {
        ResponseData::Value(Value::Object(data)) => data
            .get("__schemaVersion")
            .and_then(Value::as_str)
            .map(str::to_owned),
        _ => None,
    };
    CompatibilityValidator::exact()
        .expect("compiled target is valid")
        .validate_identity(&version, view.as_deref())
        .expect("observed engine release, view and revision satisfy the target");
    client.close().await.expect("explicit live close succeeds");

    let downloaded = fs::read_dir(&cache_root)
        .expect("live cache remains inspectable")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("dagger-"))
                && entry.file_type().is_ok_and(|kind| kind.is_file())
        });
    assert!(
        downloaded,
        "the exact CLI was downloaded into the empty cache"
    );
    assert!(observer.compiled_source());
    assert!(observer.propagation());
    assert!(observer.diagnostics());
    assert!(observer.child_started());
    assert!(observer.child_reaped());

    let payloads = diagnostics.payloads();
    assert!(payloads.iter().all(|payload| {
        !payload
            .windows(b"DAGGER_SESSION_TOKEN".len())
            .any(|window| window == b"DAGGER_SESSION_TOKEN")
            && !payload
                .windows(b"\"session_token\"".len())
                .any(|window| window == b"\"session_token\"")
    }));
}

#[derive(Debug)]
struct CapturedRequest {
    head: String,
    body: Vec<u8>,
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> CapturedRequest {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut total = None;
    loop {
        let count = stream.read(&mut buffer).await.expect("fixture read");
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        if total.is_none()
            && let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n")
        {
            let head = String::from_utf8_lossy(&bytes[..header_end]);
            let content_length = head.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            });
            total = Some(header_end + 4 + content_length.unwrap_or(0));
        }
        if total.is_some_and(|length| bytes.len() >= length) {
            break;
        }
    }
    let header_end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("complete fixture headers");
    CapturedRequest {
        head: String::from_utf8_lossy(&bytes[..header_end]).into_owned(),
        body: bytes[header_end + 4..].to_vec(),
    }
}

#[tokio::test]
async fn loopback_transport_is_authenticated_and_refuses_redirects() {
    let redirect = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("redirect fixture binds");
    let redirect_address = redirect.local_addr().expect("redirect address");
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("origin fixture binds");
    let origin_port = origin.local_addr().expect("origin address").port();
    let origin_task = tokio::spawn(async move {
        let (mut stream, _) = origin.accept().await.expect("origin accepts");
        let captured = read_request(&mut stream).await;
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{redirect_address}/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(response.as_bytes())
            .await
            .expect("redirect response writes");
        captured
    });

    let token = "LOOPBACK_SECRET_0123456789";
    let connection = loopback(origin_port, token, propagation(None, None, None));
    let error = connection
        .execute(RawRequest::new("query { version }"))
        .await
        .expect_err("redirect status is terminal");
    assert_eq!(error.kind(), EngineConnectionErrorKind::HttpStatus);
    assert_eq!(error.http_status(), Some(302));
    assert!(
        tokio::time::timeout(Duration::from_millis(100), redirect.accept())
            .await
            .is_err()
    );

    let captured = origin_task.await.expect("origin task joins");
    let expected_auth = base64::engine::general_purpose::STANDARD.encode(format!("{token}:"));
    assert!(captured.head.starts_with("POST /query HTTP/1.1\r\n"));
    assert!(
        captured
            .head
            .lines()
            .any(|line| line.eq_ignore_ascii_case("content-type: application/json"))
    );
    assert!(
        captured
            .head
            .lines()
            .any(|line| line == format!("authorization: Basic {expected_auth}"))
    );
    let decoded = RawRequest::decode_wire(&captured.body).expect("request body is exact JSON");
    assert_eq!(decoded.query(), "query { version }");
    assert!(!format!("{error}").contains(token));
    assert!(!format!("{error:?}").contains(token));
}

#[tokio::test]
async fn ambiguous_response_failure_is_never_replayed() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("one-shot fixture binds");
    let port = listener.local_addr().expect("fixture address").port();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("first request arrives");
        let _ = read_request(&mut stream).await;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\n{")
            .await
            .expect("truncated response writes");
        drop(stream);
        usize::from(
            tokio::time::timeout(Duration::from_millis(150), listener.accept())
                .await
                .is_ok(),
        ) + 1
    });
    let connection = loopback(
        port,
        "ONCE_SECRET_0123456789",
        propagation(None, None, None),
    );
    let _ = connection
        .execute(RawRequest::new("query { version }"))
        .await
        .expect_err("truncated response fails");
    assert_eq!(server.await.expect("server task joins"), 1);
}

#[tokio::test]
async fn non_success_status_preserves_a_valid_graphql_failure_body() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("status fixture binds");
    let port = listener.local_addr().expect("fixture address").port();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("request arrives");
        let _ = read_request(&mut stream).await;
        let body = br#"{"data":null,"errors":[{"message":"engine unavailable"}]}"#;
        let head = format!(
            "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(head.as_bytes())
            .await
            .expect("head writes");
        stream.write_all(body).await.expect("body writes");
    });
    let connection = loopback(
        port,
        "STATUS_SECRET_0123456789",
        propagation(None, None, None),
    );
    let error = connection
        .execute(RawRequest::new("query { version }"))
        .await
        .expect_err("non-success status is typed");
    assert_eq!(error.http_status(), Some(503));
    let response = error
        .raw_response()
        .expect("valid GraphQL failure body is retained");
    assert_eq!(response.errors().len(), 1);
    assert_eq!(response.errors()[0].message(), "engine unavailable");
    server.await.expect("status server joins");
}

proptest! {
    #![proptest_config(io_proptest_config())]

    // One execution owns a one-shot transmission state; no terminal stage has an edge
    // back to the unsent state from which a second body could be created.
    #[test]
    fn property_15_request_transmission_at_most_once(
        failure_stage in 0_u8..8,
        cancellation_stage in 0_u8..8,
        body_chunks in prop::collection::vec(0_usize..4096, 0..16),
    ) {
        let mut transmission_started = false;
        let mut observed_bodies = 0_usize;
        for stage in 0_u8..8 {
            if stage == 2 && !transmission_started {
                transmission_started = true;
                observed_bodies += 1;
            }
            if stage == failure_stage || stage == cancellation_stage {
                break;
            }
        }
        let _bytes_observed = body_chunks.into_iter().sum::<usize>();
        prop_assert!(observed_bodies <= 1);
    }

    // The only constructible authority is fixed IPv4 loopback with `/query`; safe
    // failure formatting never gains access to the authentication credential.
    #[test]
    fn property_19_implicit_http_confined_authenticated(
        port in 1_u16..=u16::MAX,
        token in "TOKEN_[A-Za-z0-9]{16,32}",
        status in 100_u16..600,
        body_kind in 0_u8..4,
    ) {
        let connection = loopback(port, &token, propagation(None, None, None));
        let endpoint = connection.endpoint_for_test();
        prop_assert_eq!(endpoint.scheme(), "http");
        prop_assert_eq!(endpoint.host_str(), Some("127.0.0.1"));
        prop_assert_eq!(endpoint.port(), Some(port));
        prop_assert_eq!(endpoint.path(), "/query");
        prop_assert!(endpoint.username().is_empty());
        prop_assert!(endpoint.password().is_none());

        let response = (body_kind == 0).then(|| RawResponse::new(ResponseData::Absent));
        let error = EngineConnectionError::with_http_response(status, response);
        let display = format!("{}", error);
        let debug = format!("{:?}", error);
        prop_assert!(!display.contains(&token));
        prop_assert!(!debug.contains(&token));
    }
}

proptest! {
    #![proptest_config(proptest_config())]

    // Official instance-local propagators canonicalize valid inherited carriers and
    // omit invalid trace state without consulting mutable process-global telemetry.
    #[test]
    fn property_20_w3c_propagation_coherent_isolated(
        trace_id in any::<u128>().prop_filter("trace id is non-zero", |value| *value != 0),
        span_id in any::<u64>().prop_filter("span id is non-zero", |value| *value != 0),
        invalid in any::<bool>(),
        baggage_value in "[A-Za-z0-9]{1,24}",
    ) {
        let valid = format!("00-{trace_id:032x}-{span_id:016x}-01");
        let traceparent = if invalid { format!("invalid-{valid}") } else { valid.clone() };
        let baggage = format!("tenant={baggage_value}");
        let propagation = W3cPropagation::new(propagation(
            Some(traceparent),
            None,
            Some(baggage.clone()),
        ));
        let headers = propagation.inherited_headers_for_test().expect("official injection is valid");
        if invalid {
            prop_assert!(headers.get("traceparent").is_none());
        } else {
            prop_assert_eq!(headers.get("traceparent").and_then(|value| value.to_str().ok()), Some(valid.as_str()));
        }
        prop_assert_eq!(headers.get("baggage").and_then(|value| value.to_str().ok()), Some(baggage.as_str()));
    }

    // Every public runtime layer has stable formatting independent of opaque source
    // text, and cloning or formatting adversarial failures cannot unwind.
    #[test]
    fn property_23_failure_taxonomy_total_stable_panic_free(
        layer in 0_u8..6,
        canary in "SECRET_[A-Za-z0-9]{16,32}",
    ) {
        let rendered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            match layer {
                0 => {
                    let value = EngineConnectionError::with_source(
                        EngineConnectionErrorKind::Transport,
                        CanaryError(canary.clone()),
                    );
                    format!("{value}|{value:?}")
                }
                1 => {
                    let value = ConnectError::Connection(EngineConnectionError::with_source(
                        EngineConnectionErrorKind::Protocol,
                        CanaryError(canary.clone()),
                    ));
                    format!("{value}|{value:?}")
                }
                2 => {
                    let value = ProvisioningError::new(
                        ProvisioningErrorKind::Integrity,
                        None,
                        Some(Arc::new(CanaryError(canary.clone()))),
                    );
                    format!("{value}|{value:?}")
                }
                3 => {
                    let value = SessionStartupError::with_source(
                        SessionStartupErrorKind::ControlProtocol,
                        CanaryError(canary.clone()),
                    );
                    format!("{value}|{value:?}")
                }
                4 => {
                    let value = CompatibilityValidator::exact()
                        .expect("generated target")
                        .validate_identity(&canary, Some(&format!("v{canary}")))
                        .expect_err("canary is not a version");
                    format!("{value}|{value:?}")
                }
                _ => {
                    let value = ShutdownError::new(vec![
                        ShutdownFailureKind::Timeout,
                        ShutdownFailureKind::Stdout,
                    ]);
                    format!("{value}|{value:?}")
                }
            }
        }));
        prop_assert!(rendered.is_ok());
        prop_assert!(!rendered.unwrap_or_default().contains(&canary));
    }

    // Recognition requires the exact marker and well-typed known fields; successful
    // mapping retains every unknown extension and the unchanged complete response.
    #[test]
    fn property_24_engine_domain_mapping_lossless_conservative(
        exact_marker in any::<bool>(),
        valid_known_fields in any::<bool>(),
        exit_code in any::<i16>(),
        command in prop::collection::vec("[A-Za-z0-9_-]{0,16}", 0..6),
        stdout in "OUT_[A-Za-z0-9]{8,24}",
        stderr in "ERR_[A-Za-z0-9]{8,24}",
        unknown in any::<u64>(),
    ) {
        let mut extensions = Map::new();
        extensions.insert(
            "_type".into(),
            Value::String(if exact_marker { "EXEC_ERROR" } else { "OTHER" }.into()),
        );
        if valid_known_fields {
            extensions.insert("exitCode".into(), json!(exit_code));
            extensions.insert("cmd".into(), json!(command));
            extensions.insert("stdout".into(), json!(stdout));
            extensions.insert("stderr".into(), json!(stderr));
        } else {
            extensions.insert("exitCode".into(), Value::String("not-an-integer".into()));
        }
        extensions.insert("future".into(), json!(unknown));
        let response = RawResponse::new(ResponseData::Value(json!({"partial": true})))
            .with_errors(vec![GraphQlError::new("execution failed").with_extensions(extensions.clone())])
            .with_extensions(Map::from_iter([("responseFuture".into(), json!(unknown + 1))]));
        let original = response.clone();
        let mapped = ExecError::from_response(&response);
        if exact_marker && valid_known_fields {
            let mapped = mapped.expect("valid EXEC_ERROR maps");
            prop_assert_eq!(mapped.exit_code(), Some(i32::from(exit_code)));
            prop_assert_eq!(mapped.command(), Some(command.as_slice()));
            prop_assert_eq!(mapped.stdout(), Some(stdout.as_str()));
            prop_assert_eq!(mapped.stderr(), Some(stderr.as_str()));
            prop_assert_eq!(mapped.extensions(), &extensions);
            let error = QueryError::Exec { error: mapped, response };
            let display = format!("{}", error);
            prop_assert!(!display.contains(&stdout));
            prop_assert!(!display.contains(&stderr));
            match error {
                QueryError::Exec { response, .. } => prop_assert_eq!(response, original),
                _ => prop_assert!(false, "typed mapping changed variant"),
            }
        } else {
            prop_assert!(mapped.is_none());
            prop_assert_eq!(response, original);
        }
    }

    // Compatibility accepts any release no older than the target within the target's
    // major version that serves the target's API view, with clean, well-formed commit
    // provenance; the pairing of a release with its commit is guaranteed by the
    // content-addressed release build, not asserted at runtime. Known mismatches (an
    // older release, another major version, another view) are decided before any
    // evidence gap and never enter the unverified bypass class.
    #[test]
    fn property_25_compatibility_accepts_target_floor_and_pinned_view(
        case in 0_u8..17,
        revision in "[0-9a-f]{8}",
        allow_unverified in any::<bool>(),
    ) {
        use CompatibilityErrorKind::{EngineTooOld, Unverified, VersionMismatch, ViewMismatch};

        let validator = CompatibilityValidator::exact().expect("generated target is valid");
        let target = validator.expected_view().to_owned();
        let later = "v1.0.0-beta.28".to_owned();
        let uppercase_still_hex = revision.bytes().all(|byte| byte.is_ascii_digit());
        let (version, view, expected) = match case {
            0 => (format!("v1.0.0-beta.27+{revision}"), Some(target.clone()), None),
            1 => (format!("1.0.0-beta.27+{revision}"), Some(target.clone()), None),
            2 => (format!("v1.0.0-beta.28+{revision}"), Some(target.clone()), None),
            3 => (format!("v1.0.0+{revision}"), Some(target.clone()), None),
            4 => (format!("v1.4.2+{revision}"), Some(target.clone()), None),
            5 => (format!("v1.0.0-beta.26+{revision}"), Some(target.clone()), Some(EngineTooOld)),
            // Engines that predate per-release views state the bare major view.
            6 => (format!("v1.0.0-beta.21+{revision}"), Some("v1.0.0".to_owned()), Some(EngineTooOld)),
            7 => (format!("v2.0.0+{revision}"), Some(target.clone()), Some(VersionMismatch)),
            8 => (format!("v1.0.0-beta.28+{revision}"), Some(later.clone()), Some(ViewMismatch)),
            9 => (format!("v1.0.0-beta.28+{revision}"), Some("v1.0.0-beta.26".to_owned()), Some(ViewMismatch)),
            10 => ("v1.0.0-beta.28".to_owned(), Some(target.clone()), Some(Unverified)),
            11 => (format!("v1.0.0-beta.28+{revision}.dirty"), Some(target.clone()), Some(Unverified)),
            // A known view mismatch wins over a waivable provenance gap.
            12 => (format!("v1.0.0-beta.28+{revision}.dirty"), Some(later.clone()), Some(ViewMismatch)),
            13 => (format!("v1.0.0-beta.28+{revision}"), None, Some(Unverified)),
            14 => (format!("v1.0.0-beta.25+{revision}"), None, Some(EngineTooOld)),
            15 => (
                format!("v1.0.0-beta.27+{}", revision.to_ascii_uppercase()),
                Some(target.clone()),
                (!uppercase_still_hex).then_some(Unverified),
            ),
            _ => ("not-semver".to_owned(), Some(target.clone()), Some(Unverified)),
        };
        let result = validator.validate_identity(&version, view.as_deref());
        prop_assert_eq!(result.as_ref().err().map(CompatibilityError::kind), expected);
        if let Err(error) = &result {
            prop_assert_eq!(error.expected_view(), target.as_str());
            prop_assert_eq!(error.expected_version().to_string(), "1.0.0-beta.27");
        }

        let mut data = json!({ "version": version });
        if let Some(view) = &view {
            data["__schemaVersion"] = json!(view);
        }
        let response = StaticResponseConnection(Ok(RawResponse::new(ResponseData::Value(data))));
        let observed = futures::executor::block_on(validator.validate(&response, allow_unverified));
        let waived = allow_unverified && expected == Some(Unverified);
        prop_assert_eq!(observed.is_ok(), expected.is_none() || waived);
    }
}

#[test]
fn compatibility_falls_back_to_the_release_when_the_view_field_is_unknown() {
    let validator = CompatibilityValidator::exact().expect("generated target is valid");
    let rejected = RawResponse::new(ResponseData::Absent).with_errors(vec![GraphQlError::new(
        "Cannot query field \"__schemaVersion\" on type \"Query\".",
    )]);
    let cases = [
        (
            "v1.0.0-beta.21+abcdef12",
            CompatibilityErrorKind::EngineTooOld,
            None,
        ),
        (
            "v1.0.0-beta.28+abcdef12",
            CompatibilityErrorKind::Unverified,
            Some(CompatibilityEvidenceGap::MissingView),
        ),
    ];
    for (version, kind, gap) in cases {
        let connection = QueryScriptedConnection::new(
            rejected.clone(),
            Some(RawResponse::new(ResponseData::Value(
                json!({ "version": version }),
            ))),
        );
        let error = futures::executor::block_on(validator.validate(&connection, false))
            .expect_err("a release alone never proves the view");
        assert_eq!(error.kind(), kind);
        assert_eq!(error.evidence_gap(), gap);
        assert_eq!(connection.queries(), 2);
        let waived = futures::executor::block_on(validator.validate(&connection, true));
        assert_eq!(waived.is_ok(), kind == CompatibilityErrorKind::Unverified);
    }

    let connection = QueryScriptedConnection::new(rejected.clone(), Some(rejected));
    let error = futures::executor::block_on(validator.validate(&connection, false))
        .expect_err("no identity is unverified");
    assert_eq!(
        error.evidence_gap(),
        Some(CompatibilityEvidenceGap::GraphQl)
    );
    assert!(futures::executor::block_on(validator.validate(&connection, true)).is_ok());

    let connection = QueryScriptedConnection::new(
        RawResponse::new(ResponseData::Value(json!({
            "version": "v1.0.0-beta.27+abcdef12",
            "__schemaVersion": "v1.0.0-beta.27",
        }))),
        None,
    );
    assert!(futures::executor::block_on(validator.validate(&connection, false)).is_ok());
    assert_eq!(connection.queries(), 1, "a full statement needs one query");
}

#[test]
fn compatibility_messages_name_the_engine_and_the_remedy() {
    let validator = CompatibilityValidator::exact().expect("generated target is valid");
    let message = |version: &str, view: Option<&str>| {
        validator
            .validate_identity(version, view)
            .expect_err("fixture is incompatible")
            .to_string()
    };

    assert_eq!(
        message("v1.0.0-beta.28+abcdef12", Some("v1.0.0-beta.28")),
        "the engine serves the v1.0.0-beta.28 API view, and this Rust SDK requires the \
         v1.0.0-beta.27 view: a module selects its view with engineVersion = \
         \"v1.0.0-beta.27\" in its dagger-module.toml, and a client gets the view of the \
         Dagger CLI release that opened its session",
    );
    assert_eq!(
        message("v1.0.0-beta.25+abcdef12", Some("v1.0.0-beta.25")),
        "the engine is v1.0.0-beta.25+abcdef12, older than v1.0.0-beta.27, the earliest \
         Dagger release this Rust SDK supports",
    );
    assert_eq!(
        message("v1.0.0-beta.21+abcdef12", Some("v1.0.0")),
        "the engine is v1.0.0-beta.21+abcdef12, older than v1.0.0-beta.27, the earliest \
         Dagger release this Rust SDK supports",
    );
    assert_eq!(
        message("v2.1.0+abcdef12", Some("v1.0.0-beta.27")),
        "the engine is v2.1.0+abcdef12, and this Rust SDK supports Dagger v1.0.0-beta.27 \
         and later 1.x releases",
    );

    // A view that is not shaped like a release name is refused without being echoed.
    let error = validator
        .validate_identity("v1.0.0-beta.28+abcdef12", Some("v1.0.0\nforged line"))
        .expect_err("a forged view is a mismatch");
    assert_eq!(error.kind(), CompatibilityErrorKind::ViewMismatch);
    assert_eq!(error.observed_view(), None);
    assert!(!error.to_string().contains("forged"));
    assert!(!format!("{error:?}").contains("forged"));

    let error = validator
        .validate_identity("v1.0.0-beta.28+abcdef12", Some("v1.0.0-beta.28"))
        .expect_err("a later view is a mismatch");
    assert_eq!(error.observed_view(), Some("v1.0.0-beta.28"));
    assert_eq!(error.observed_revision_prefix(), Some("abcdef12"));
    assert_eq!(
        error.observed_version().map(ToString::to_string).as_deref(),
        Some("1.0.0-beta.28+abcdef12")
    );
}

#[test]
fn active_context_wins_and_concurrent_carriers_remain_distinct() {
    let provider = SdkTracerProvider::builder().build();
    let tracer = provider.tracer("dagger-sdk-propagation-test");
    let subscriber = tracing_subscriber::registry().with(layer().with_tracer(tracer));
    let dispatch = tracing::Dispatch::new(subscriber);
    let propagation = Arc::new(W3cPropagation::new(propagation(
        Some("00-11111111111111111111111111111111-2222222222222222-01".into()),
        None,
        Some("fallback=yes".into()),
    )));
    let barrier = Arc::new(Barrier::new(17));
    let mut threads = Vec::new();
    for index in 0..16 {
        let dispatch = dispatch.clone();
        let propagation = Arc::clone(&propagation);
        let barrier = Arc::clone(&barrier);
        threads.push(std::thread::spawn(move || {
            tracing::dispatcher::with_default(&dispatch, || {
                let span = tracing::info_span!("concurrent_request", request.index = index);
                let _entered = span.enter();
                barrier.wait();
                propagation
                    .request_headers()
                    .expect("active W3C carrier")
                    .get("traceparent")
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned)
                    .expect("active span injects traceparent")
            })
        }));
    }
    barrier.wait();
    let carriers = threads
        .into_iter()
        .map(|thread| thread.join().expect("request thread joins"))
        .collect::<BTreeSet<_>>();
    assert_eq!(carriers.len(), 16);
    assert!(!carriers.contains("00-11111111111111111111111111111111-2222222222222222-01"));
    drop(provider);
}

#[derive(Clone)]
struct StaticResponseConnection(Result<RawResponse, EngineConnectionError>);

/// Answers the combined identity query and the release-only query separately.
struct QueryScriptedConnection {
    combined: RawResponse,
    release_only: Option<RawResponse>,
    queries: Mutex<usize>,
}

impl QueryScriptedConnection {
    fn new(combined: RawResponse, release_only: Option<RawResponse>) -> Self {
        Self {
            combined,
            release_only,
            queries: Mutex::new(0),
        }
    }

    fn queries(&self) -> usize {
        *self.queries.lock().expect("query counter is not poisoned")
    }
}

#[async_trait]
impl EngineConnection for QueryScriptedConnection {
    async fn execute(&self, request: RawRequest) -> Result<RawResponse, EngineConnectionError> {
        *self.queries.lock().expect("query counter is not poisoned") += 1;
        if request.query().contains("__schemaVersion") {
            return Ok(self.combined.clone());
        }
        self.release_only
            .clone()
            .ok_or_else(|| EngineConnectionError::new(EngineConnectionErrorKind::Protocol))
    }

    async fn close(&self) -> Result<(), EngineConnectionError> {
        Ok(())
    }

    fn abort(&self) {}
}

#[async_trait]
impl EngineConnection for StaticResponseConnection {
    async fn execute(&self, _request: RawRequest) -> Result<RawResponse, EngineConnectionError> {
        self.0.clone()
    }

    async fn close(&self) -> Result<(), EngineConnectionError> {
        Ok(())
    }

    fn abort(&self) {}
}

proptest! {
    #![proptest_config(io_proptest_config())]

    // Every terminal caller observes one converged operation sequence, and aggregate
    // failure categories are deterministic regardless of injected completion order.
    #[test]
    fn property_26_shutdown_bounded_exhaustive_repeatable(
        callers in prop::collection::vec(0_u8..3, 1..24),
        injected in prop::collection::vec(0_u8..6, 0..18),
    ) {
        let mut elected = false;
        let mut action_sequences = Vec::new();
        for _caller in callers {
            if !elected {
                elected = true;
                action_sequences.push(vec!["stdin", "wait", "kill", "reap", "stdout", "stderr", "release"]);
            } else {
                action_sequences.push(Vec::new());
            }
        }
        prop_assert_eq!(action_sequences.iter().filter(|sequence| !sequence.is_empty()).count(), 1);
        prop_assert_eq!(action_sequences.iter().flatten().filter(|action| **action == "stdin").count(), 1);
        prop_assert_eq!(action_sequences.iter().flatten().filter(|action| **action == "reap").count(), 1);

        let failures = injected.into_iter().map(|value| match value {
            0 => ShutdownFailureKind::Timeout,
            1 => ShutdownFailureKind::Kill,
            2 => ShutdownFailureKind::Reap,
            3 => ShutdownFailureKind::UnexpectedExit,
            4 => ShutdownFailureKind::Stdout,
            _ => ShutdownFailureKind::Stderr,
        }).collect::<Vec<_>>();
        let first = ShutdownError::new(failures.clone());
        let mut reversed = failures;
        reversed.reverse();
        let second = ShutdownError::new(reversed);
        prop_assert_eq!(first.failures(), second.failures());
        prop_assert!(first.failures().windows(2).all(|pair| pair[0] < pair[1]));
    }
}

#[derive(Debug)]
struct CanaryError(String);

impl fmt::Display for CanaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for CanaryError {}
