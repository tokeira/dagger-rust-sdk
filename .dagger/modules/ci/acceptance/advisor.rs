//! A module whose agent function returns its advice instead of the conversation it
//! extends, which the engine refuses when it loads the module.

pub mod dagger_generated;

#[dagger_sdk::object(root, rename = "Advisor")]
pub struct Advisor {}

#[dagger_sdk::methods]
impl Advisor {
    #[dagger(constructor, rename = "advisor")]
    pub fn new() -> Advisor {
        Advisor {}
    }

    /// Returns advice, where an agent function returns the conversation.
    #[dagger(function, role = "agent")]
    pub fn expert(&self, base: dagger_sdk::Llm) -> String {
        drop(base);
        "check the release".to_owned()
    }
}
