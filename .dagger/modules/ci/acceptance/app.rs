//! Module-clients acceptance: a Cargo package that is not a module calls two modules
//! through the clients `dagger module client add` generated into `src/dagger_clients`.

mod dagger_clients;

use std::error::Error;

use dagger_clients::{greeter, naming_app};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = dagger_sdk::connect().await?;

    // The greeter comes from a Git repository: its constructor with the optional
    // argument, and one function.
    let greeting = greeter::Client::new(&client)
        .greeter_opts(&greeter::QueryGreeterOpts::default().with_greeting("hi"))
        .hello("app")
        .await?;

    // naming-app is in the workspace. The engine serves its fetcher as
    // `NamingAppHTTPFetcher`, the name that loading it again by its ID must send.
    let modules = naming_app::Client::new(&client);
    let id = modules
        .naming_app()
        .fetcher("https://example.invalid")
        .id()
        .await?;
    let fetcher: naming_app::NamingAppHttpFetcher = modules.load(id).await?;
    let url = fetcher.url().await?;
    let retries = fetcher.retries().await?;

    client.close().await?;
    println!("ACCEPTANCE-MODULE-CLIENTS-OK {greeting}; {url} {retries}");
    Ok(())
}
