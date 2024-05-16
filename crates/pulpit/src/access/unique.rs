//! ## Storage of a unique index
//! Allows for fast access, as well as checking unique constrains.
//! 
//! In order to 
//! ## Efficient Design
//! An O(1) set of `Wrapper<Key>`, which borrows from the table on access.
//! - No extra data kept
//! - Can simply access and if valid, convert value to the contained key
//! - efficient recompute of the index on update, simply pass index to update. 
//! 
//! The same benefits for immutability as in [`crate::column`] can be applied here.
//! 
//! ## Correct Design
//! To save dev time: just `get` the value (cringe but easy).

use std::{collections::HashMap, hash::Hash};
 
struct MissingValue;
struct Conflict;

/// A simple wrapper for storing copies of keys and associated unique values in 
/// an index.
struct Unique<Field, Key> {
    mapping: HashMap<Field, Key>,
}

impl <Field: Eq + Hash, Key: Copy> Unique<Field, Key> {
    fn new(size_hint: usize) -> Self {
        Self { mapping: HashMap::with_capacity(size_hint) }
    }

    fn lookup(&self, value: &Field) -> Result<Key, MissingValue> {
        match self.mapping.get(value) {
            Some(k) => Ok(*k),
            None => Err(MissingValue),
        }
    }

    fn insert(&mut self, field: Field, key: Key) -> Result<(), Conflict> {
        match self.mapping.insert(field, key) {
            Some(_) => Err(Conflict),
            None => Ok(()),
        }
    }

    fn pull(&mut self, field: Field) -> Result<(), MissingValue> {
        match self.mapping.remove(&field) {
            Some(_) => Ok(()),
            None => Err(MissingValue),
        }
    }
}
