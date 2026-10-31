//! Standalone Dagger client composed over the shared public Rust SDK runtime.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:e0fa5b0e88fd7137b64e4af00205e21391e92aa630b8bd882b594fc4b32c6a6b","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
pub use dagger_sdk as core;
pub use dagger_sdk::{Client, ClientConfig, connect, connect_with};
mod generated;
/// Typed bindings for selected GraphQL module `minimal`.
#[path = "generated/minimal/mod.rs"]
pub mod minimal;
/// Adds the selected GraphQL module root to an existing shared client.
pub trait MinimalExt {
    /// Selects GraphQL root field `minimal` without opening another session.
    fn minimal(&self) -> minimal::Client;
}
impl MinimalExt for dagger_sdk::Client {
    fn minimal(&self) -> minimal::Client {
        minimal::Client::from_query(self.query_builder().select("minimal"))
    }
}
impl MinimalExt for dagger_sdk::QueryBuilder {
    fn minimal(&self) -> minimal::Client {
        minimal::Client::from_query(self.select("minimal"))
    }
}
/// Imports the selected module extension trait for method resolution.
pub mod prelude {
    pub use super::MinimalExt as _;
}
