//! Standalone Dagger client composed over the shared public Rust SDK runtime.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
