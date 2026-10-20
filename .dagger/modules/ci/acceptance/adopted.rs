//! An existing crate's source, which `dagger module init` adopts as it is.

// Declared by the crate itself: initialization never edits authored source.
pub mod dagger_generated;

#[dagger_sdk::object(root, rename = "Existing")]
pub struct Existing {}

#[dagger_sdk::methods]
impl Existing {
    #[dagger(constructor, rename = "existing")]
    pub fn new() -> Existing {
        Existing {}
    }

    /// Says that the crate was adopted.
    #[dagger(function)]
    pub fn adopted(&self) -> String {
        "adopted".to_owned()
    }
}
