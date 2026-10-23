//! Typed bindings for GraphQL module root `minimal`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:aedcb1007e675ae595d3bc7316118dcd81bbece3291c5b3ce518f47f66cc9cbf","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
