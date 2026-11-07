//! Standalone Dagger client composed over the shared public Rust SDK runtime.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:506c57349967eeaceb1235a4465193229f99ff7b15c2a58a4e8fd04be3d5008f","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
