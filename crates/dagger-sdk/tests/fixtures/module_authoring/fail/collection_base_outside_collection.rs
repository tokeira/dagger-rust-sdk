mod dagger_generated {
    pub mod __private {
        #[allow(unused_imports)]
        pub use dagger_sdk::__private::*;
    }
}

#[dagger_sdk::object]
pub struct Items {
    #[dagger(field)]
    names: Vec<String>,
    base: dagger_sdk::CollectionBase,
}

fn main() {}
