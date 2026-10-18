//! An application outside this repository that depends on the SDK as the crate's README
//! says to: by version alone. `package` builds it against the release's archives before
//! they are published, and `published` against crates.io afterwards. It expands the
//! authoring macros through the SDK, then makes real calls to the engine it runs under.

use dagger_sdk::ModuleError;

// The authoring macros expand against the code `dagger generate` writes into a module;
// this stands in for it, as in the SDK's own compile tests.
mod dagger_generated {
    pub mod __private {
        pub use dagger_sdk::__private::*;
    }
}

/// A module object, so the macros crate is exercised through the SDK's re-exports.
#[dagger_sdk::object(root, rename = "Consumer")]
pub struct Consumer {
    #[dagger(field)]
    greeting: String,
}

#[dagger_sdk::methods]
impl Consumer {
    #[dagger(constructor)]
    fn new(greeting: String) -> Self {
        Self { greeting }
    }

    #[dagger(function)]
    fn greet(&self, #[dagger(default = "world")] name: String) -> Result<String, ModuleError> {
        Ok(format!("{} {name}", self.greeting))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let greeting = Consumer::new("hello".to_owned()).greet("consumer".to_owned())?;
    let client = dagger_sdk::connect().await?;
    let query = client.query();
    let engine = query.version().await?;
    let echoed = query
        .container()
        .from("alpine:3.22")
        .with_exec(vec!["echo", greeting.as_str()])
        .stdout()
        .await?;
    client.close().await?;
    println!("CONSUMER-OK {engine} {}", echoed.trim_end());
    Ok(())
}
