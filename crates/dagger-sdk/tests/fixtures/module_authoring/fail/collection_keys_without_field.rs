mod dagger_generated {
    pub mod __private {
        #[allow(unused_imports)]
        pub use dagger_sdk::__private::*;
    }
}

#[dagger_sdk::object(collection)]
pub struct Items {
    #[dagger(keys)]
    names: Vec<String>,
    base: dagger_sdk::CollectionBase,
}

fn main() {}
