//! Typed bindings for GraphQL module root `minimal`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:e0fa5b0e88fd7137b64e4af00205e21391e92aa630b8bd882b594fc4b32c6a6b","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[path = "client.rs"]
mod client;
pub use client::*;
#[path = "config.rs"]
mod config;
pub use config::*;
#[path = "item.rs"]
mod item;
pub use item::*;
#[path = "minimal_client.rs"]
mod minimal_client;
pub use minimal_client::*;
#[path = "node.rs"]
mod node;
pub use node::*;
#[path = "state.rs"]
mod state;
pub use state::*;
#[path = "token.rs"]
mod token;
pub use token::*;
