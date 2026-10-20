//! A module configured by a legacy `dagger.json`, whose code the runtime generates in a
//! private snapshot rather than in the workspace.

// Declared by the module; the runtime generates its contents for each load.
pub mod dagger_generated;

#[dagger_sdk::object(root, rename = "Legacy")]
pub struct Legacy {}

#[dagger_sdk::methods]
impl Legacy {
    #[dagger(constructor, rename = "legacy")]
    pub fn new() -> Legacy {
        Legacy {}
    }

    /// Answers a call without any committed generated code.
    #[dagger(function)]
    pub fn probe(&self) -> String {
        "rust-sdk-protocol-ok".to_owned()
    }
}
