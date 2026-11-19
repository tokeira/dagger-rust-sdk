//! A module whose function takes an object of the module it depends on, which the engine
//! refuses when it loads the module.

pub mod dagger_generated;

#[dagger_sdk::object(root, rename = "Lender")]
pub struct Lender {}

#[dagger_sdk::methods]
impl Lender {
    #[dagger(constructor, rename = "lender")]
    pub fn new() -> Lender {
        Lender {}
    }

    /// Greets a name with a greeter of the caller's, where a function may take only Core's
    /// types and the module's own.
    #[dagger(function)]
    pub async fn greet(
        &self,
        greeter: dagger_generated::Greeter,
        name: String,
    ) -> Result<String, dagger_sdk::ModuleError> {
        Ok(greeter.hello(name).await?)
    }
}
