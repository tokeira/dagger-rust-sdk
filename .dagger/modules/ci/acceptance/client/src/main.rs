//! Plain-client acceptance: connect, make Void calls, and export an image.

use std::error::Error;

use dagger_sdk::EngineCachePruneOpts;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = dagger_sdk::connect().await?;
    let version = client.query().version().await?;
    // The engine's own collection policy, so a shared engine keeps the cache it would keep.
    client
        .query()
        .engine()
        .local_cache()
        .prune_opts(&EngineCachePruneOpts::default().with_use_default_policy(true))
        .await?;
    client.query().current_workspace().export().await?;
    client
        .query()
        .container()
        .with_new_file("void canary", "/rust-sdk-acceptance-canary")
        .export("canary.tar")
        .await?;
    client.close().await?;
    println!("ACCEPTANCE-CLIENT-OK {version}");
    Ok(())
}
