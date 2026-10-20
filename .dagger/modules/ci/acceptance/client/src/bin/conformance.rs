//! Core-conformance acceptance: each kind of generated Core operation, and each kind of
//! failure the client reports, against a live engine.

use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use dagger_sdk::{
    BuildArg, CacheSharingMode, ClientConfig, DirectoryDockerBuildOpts, EngineCachePruneOpts,
    EngineConnection, EngineConnectionError, EngineConnectionErrorKind, QueryCacheVolumeOpts,
    QueryError, QueryGitOpts, RawRequest, RawResponse, RequestError, ResponseData, Syncer,
};
use serde_json::json;

/// A connection that fails the way `ProbeMode` names, without an engine.
struct ProbeConnection(ProbeMode);

enum ProbeMode {
    /// Every request fails in transport.
    Transport,
    /// Every request answers with a value no generated type decodes.
    Decode,
}

#[async_trait]
impl EngineConnection for ProbeConnection {
    async fn execute(&self, _request: RawRequest) -> Result<RawResponse, EngineConnectionError> {
        match self.0 {
            ProbeMode::Transport => Err(EngineConnectionError::new(
                EngineConnectionErrorKind::Transport,
            )),
            ProbeMode::Decode => Ok(RawResponse::new(ResponseData::Value(json!({
                "version": 42
            })))),
        }
    }

    async fn close(&self) -> Result<(), EngineConnectionError> {
        Ok(())
    }

    fn abort(&self) {}
}

async fn probe_client(mode: ProbeMode) -> Result<dagger_sdk::Client, Box<dyn Error>> {
    let config = ClientConfig::builder()
        .connection(Box::new(ProbeConnection(mode)))
        .build()?;
    Ok(dagger_sdk::connect_with(config).await?)
}

async fn interface_id(value: &impl Syncer) -> Result<dagger_sdk::Id, QueryError> {
    Syncer::id(value).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = dagger_sdk::connect().await?;
    let query = client.query();

    // A scalar and a custom scalar.
    let version = query.version().await?;
    let _platform = query.default_platform().await?;

    // An enum argument.
    let enum_opts = QueryCacheVolumeOpts::default().with_sharing(CacheSharingMode::Private);
    let _cache_id = query
        .cache_volume_opts("rust-core-conformance-enum", &enum_opts)
        .id()
        .await?;

    // An input-object argument.
    let dockerfile = "FROM alpine:3.22\nARG MESSAGE\nRUN test \"$MESSAGE\" = input-object\n";
    let source = query.directory().with_new_file(dockerfile, "Dockerfile");
    let build_opts = DirectoryDockerBuildOpts::default().with_build_args(vec![BuildArg::new(
        "MESSAGE".to_owned(),
        "input-object".to_owned(),
    )]);
    let built = source.docker_build_opts(&build_opts);
    assert_eq!(
        built
            .with_exec(vec!["sh", "-c", "printf input-object"])
            .stdout()
            .await?,
        "input-object"
    );

    // A lazy object, its ID through an interface, and a nullable handle.
    let lazy = query.container().from("alpine:3.22");
    let _interface_id = interface_id(&lazy).await?;
    assert!(lazy.docker_healthcheck().await?.is_none());

    // A list of objects, in order.
    let envs = lazy
        .with_env_variable("RUST_CONFORMANCE_FIRST", "one")
        .with_env_variable("RUST_CONFORMANCE_SECOND", "two")
        .env_variables()
        .await?;
    let mut names = Vec::with_capacity(envs.len());
    for env in envs {
        names.push(env.name().await?);
    }
    let first = names
        .iter()
        .position(|name| name == "RUST_CONFORMANCE_FIRST")
        .expect("first environment value must be retained");
    let second = names
        .iter()
        .position(|name| name == "RUST_CONFORMANCE_SECOND")
        .expect("second environment value must be retained");
    assert!(first < second);

    // An ID of any type, and a handle where an ID is expected.
    let lazy_id = lazy.id().await?;
    assert!(query.node(lazy_id).await?.is_some());
    let handle_opts = QueryCacheVolumeOpts::default().with_source(
        query
            .directory()
            .with_new_file("retained", "handle-source")
            .into(),
    );
    let _handle_id = query
        .cache_volume_opts("rust-core-conformance-handle", &handle_opts)
        .id()
        .await?;

    // A call that returns its own object.
    let synced = lazy.sync().await?;
    let _synced_id = synced.id().await?;

    // A zero-like value, sent rather than omitted.
    let git_opts = QueryGitOpts::default().with_keep_git_dir(false);
    let commit = query
        .git_opts("https://github.com/octocat/Hello-World.git", &git_opts)
        .head()
        .commit_sha()
        .await?;
    assert!(!commit.is_empty());

    // A GraphQL error and a failed process.
    let graphql_error = query
        .directory()
        .with_error("rust core conformance GraphQL error")
        .entries()
        .await
        .expect_err("withError must surface a GraphQL failure");
    assert!(matches!(graphql_error, QueryError::GraphQl { .. }));
    let exec_error = query
        .container()
        .from("alpine:3.22")
        .with_exec(vec!["sh", "-c", "exit 17"])
        .stdout()
        .await
        .expect_err("non-zero process exit must surface an engine-domain error");
    assert!(matches!(exec_error, QueryError::Exec { .. }));

    // A request that outlives the client's timeout. The value no other run used keeps the
    // engine from answering it from its cache.
    let run = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let timeout_config = ClientConfig::builder()
        .graphql_execution_timeout(Duration::from_millis(250))
        .build()?;
    let timeout_client = dagger_sdk::connect_with(timeout_config).await?;
    let timeout_error = timeout_client
        .query()
        .container()
        .from("alpine:3.22")
        .with_env_variable("RUST_CONFORMANCE_RUN", run.to_string())
        .with_exec(vec!["sh", "-c", "sleep 2"])
        .stdout()
        .await
        .expect_err("the complete-request timeout must fence a slow generated call");
    assert!(matches!(
        timeout_error,
        QueryError::Request(RequestError::ExecutionTimeout { .. })
    ));
    timeout_client.close().await?;

    // A transport failure, and a response no generated type decodes.
    let transport_client = probe_client(ProbeMode::Transport).await?;
    let transport_error = transport_client
        .query()
        .version()
        .await
        .expect_err("the injected transport failure must remain typed");
    assert!(matches!(
        transport_error,
        QueryError::Request(RequestError::Connection(ref error))
            if error.kind() == EngineConnectionErrorKind::Transport
    ));
    transport_client.close().await?;
    let decode_client = probe_client(ProbeMode::Decode).await?;
    let decode_error = decode_client
        .query()
        .version()
        .await
        .expect_err("the incompatible response value must fail generated decoding");
    assert!(matches!(decode_error, QueryError::Decode(_)));
    decode_client.close().await?;

    // A Void call, under the engine's own collection policy, so a shared engine keeps the
    // cache it would keep.
    query
        .engine()
        .local_cache()
        .prune_opts(&EngineCachePruneOpts::default().with_use_default_policy(true))
        .await?;

    // A call after close, which the client refuses itself.
    client.close().await?;
    let close_error = query
        .version()
        .await
        .expect_err("generated calls after close must be fenced locally");
    assert!(matches!(
        close_error,
        QueryError::Request(RequestError::ClientClosed)
    ));

    println!("ACCEPTANCE-CONFORMANCE-OK {version}");
    Ok(())
}
