//! A crate whose author already wrote a file at a generated path, `src/bin/dagger-module.rs`.

pub mod dagger_generated;

#[dagger_sdk::object(root, rename = "Collision")]
pub struct Collision {}

#[dagger_sdk::methods]
impl Collision {
    #[dagger(constructor, rename = "collision")]
    pub fn new() -> Collision {
        Collision {}
    }
}
