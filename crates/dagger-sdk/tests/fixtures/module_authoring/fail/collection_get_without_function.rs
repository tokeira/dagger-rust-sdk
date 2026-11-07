mod dagger_generated {
    pub mod __private {
        #[allow(unused_imports)]
        pub use dagger_sdk::__private::*;
    }
}

#[dagger_sdk::object(collection)]
pub struct Items {
    #[dagger(field)]
    keys: Vec<String>,
    base: dagger_sdk::CollectionBase,
}

#[dagger_sdk::methods]
impl Items {
    #[dagger(get)]
    fn get(&self, key: String) -> String {
        key
    }
}

fn main() {}
