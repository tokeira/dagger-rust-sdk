//! Typed bindings for GraphQL module root `minimal`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:af5c520c5d560afb009cab87b651624ea148759019adbac1610ddcefcc3cee49","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
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
