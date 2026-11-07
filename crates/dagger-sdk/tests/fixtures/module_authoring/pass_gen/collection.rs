use dagger_sdk as renamed_sdk;

mod dagger_generated {
    pub mod __private {
        pub use crate::renamed_sdk::__private::*;
    }
}

// The base may sit anywhere among the fields; the persistent state carries it last.
#[renamed_sdk::object(collection, rename = "items")]
#[derive(Clone)]
pub(crate) struct Items {
    #[dagger(field, keys)]
    names: Vec<String>,
    base: renamed_sdk::CollectionBase,
    #[dagger(field, delta)]
    selection: Option<renamed_sdk::CollectionDelta>,
    #[dagger(state)]
    prefix: String,
    transient_counter: usize,
}

#[renamed_sdk::object(rename = "item")]
pub(crate) struct Item {
    #[dagger(field)]
    name: String,
}

#[renamed_sdk::methods]
impl Items {
    #[dagger(function, get)]
    fn lookup(&self, name: String) -> Item {
        Item {
            name: format!("{}{name}", self.prefix),
        }
    }

    #[dagger(function)]
    fn with_prefix(&self, prefix: String) -> Items {
        Items {
            prefix,
            ..self.clone()
        }
    }

    #[dagger(function)]
    fn fresh(&self) -> Items {
        Items {
            names: vec!["z".to_owned()],
            base: renamed_sdk::CollectionBase::default(),
            selection: None,
            prefix: String::new(),
            transient_counter: self.transient_counter,
        }
    }
}

fn require_expected_state<T>()
where
    T: renamed_sdk::__private::ModuleObjectBridge<
            PersistentState = (
                Vec<String>,
                Option<renamed_sdk::CollectionDelta>,
                String,
                renamed_sdk::CollectionBase,
            ),
        >,
{
}

fn main() {
    require_expected_state::<Items>();
}
