//! The base that a collection value carries between the engine and a module.

/// The member of a collection's JSON that names the collection it was derived from.
const BASE_MEMBER: &str = "__daggerCollectionBase";

/// The collection that a collection value was derived from.
///
/// When the engine hands a module a collection, as a receiver, an argument, or a value
/// nested in one, the collection names the collection it was derived from: its base. The
/// engine computes a collection's delta against its base, and applies `dagger check`'s item
/// selection to a derived collection through it. Each collection object declares one field
/// of this type, which the engine fills. A value built from a received collection, by
/// `self.clone()` or by `..self.clone()`, keeps the base. A fresh value starts from
/// `CollectionBase::default()`, so the engine treats it as its own base.
///
/// ```
/// # use dagger_sdk::CollectionBase;
/// let fresh = CollectionBase::default();
/// assert_eq!(fresh.clone(), fresh);
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CollectionBase(Option<String>);

#[doc(hidden)]
pub mod __private {
    use serde_json::{Map, Value};

    use super::{BASE_MEMBER, CollectionBase};
    use crate::module::codec::__private::DecodeError;

    /// Reads the base from a collection's JSON. An absent, null, or empty member means the
    /// value is its own base, as it does for the engine.
    pub fn decode_collection_base(
        object: &Map<String, Value>,
    ) -> Result<CollectionBase, DecodeError> {
        match object.get(BASE_MEMBER) {
            None | Some(Value::Null) => Ok(CollectionBase(None)),
            Some(Value::String(base)) if base.is_empty() => Ok(CollectionBase(None)),
            Some(Value::String(base)) => Ok(CollectionBase(Some(base.clone()))),
            Some(_) => Err(DecodeError),
        }
    }

    /// Writes a derived value's base into its JSON, and leaves a fresh value's out.
    pub fn encode_collection_base(base: CollectionBase, object: &mut Map<String, Value>) {
        if let Some(base) = base.0 {
            object.insert(BASE_MEMBER.to_owned(), Value::String(base));
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::__private::{decode_collection_base, encode_collection_base};
    use super::CollectionBase;
    use crate::module::codec::__private::DecodeError;

    fn object(value: Value) -> Map<String, Value> {
        value.as_object().expect("an object").clone()
    }

    #[test]
    fn a_derived_value_returns_the_base_it_received() {
        let received = object(json!({"names": ["a"], "__daggerCollectionBase": "base-id"}));
        let base = decode_collection_base(&received).expect("a string base decodes");
        assert_ne!(base, CollectionBase::default());
        let mut returned = object(json!({"names": ["b"]}));
        encode_collection_base(base.clone(), &mut returned);
        assert_eq!(
            returned.get("__daggerCollectionBase"),
            Some(&json!("base-id"))
        );
    }

    #[test]
    fn a_fresh_value_returns_no_base() {
        let mut returned = object(json!({"names": ["z"]}));
        encode_collection_base(CollectionBase::default(), &mut returned);
        assert!(!returned.contains_key("__daggerCollectionBase"));
    }

    #[test]
    fn an_absent_null_or_empty_base_means_none() {
        for received in [
            json!({}),
            json!({"__daggerCollectionBase": null}),
            json!({"__daggerCollectionBase": ""}),
        ] {
            assert_eq!(
                decode_collection_base(&object(received)),
                Ok(CollectionBase::default())
            );
        }
    }

    #[test]
    fn a_base_that_is_not_a_string_is_refused() {
        assert_eq!(
            decode_collection_base(&object(json!({"__daggerCollectionBase": 1}))),
            Err(DecodeError)
        );
    }
}
