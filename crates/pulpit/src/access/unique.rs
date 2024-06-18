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

use rustc_hash::FxHashMap;
use std::hash::{BuildHasherDefault, Hash};
#[derive(Debug)]
pub struct MissingUniqueValue;
#[derive(Debug)]
pub struct UniqueConflict;

/// A simple wrapper for storing copies of keys and associated unique values in
/// an index.
pub struct Unique<Field, Key> {
    mapping: FxHashMap<Field, Key>,
}

impl<Field: Eq + Hash + Clone, Key: Copy + Eq> Unique<Field, Key> {
    pub fn new(size_hint: usize) -> Self {
        Self {
            mapping: FxHashMap::with_capacity_and_hasher(size_hint, BuildHasherDefault::default()),
        }
    }

    pub fn lookup(&self, value: &Field) -> Result<Key, MissingUniqueValue> {
        match self.mapping.get(value) {
            Some(k) => Ok(*k),
            None => Err(MissingUniqueValue),
        }
    }

    // TODO: avoid copies
    pub fn insert(&mut self, field: Field, key: Key) -> Result<(), UniqueConflict> {
        match self.mapping.insert(field.clone(), key) {
            Some(old_key) => {
                *self.mapping.get_mut(&field).unwrap() = old_key;
                Err(UniqueConflict)
            }
            None => Ok(()),
        }
    }

    pub fn pull(&mut self, field: &Field) -> Result<(), MissingUniqueValue> {
        match self.mapping.remove(field) {
            Some(_) => Ok(()),
            None => Err(MissingUniqueValue),
        }
    }

    /// At the given key, with the given old value, replace with the new value in to_insert.
    /// - If error on uniqueconflict
    /// - Otherwise returns the old value
    pub fn replace(
        &mut self,
        to_insert: &Field,
        replace: &Field,
        key: Key,
    ) -> Result<Field, UniqueConflict> {
        if to_insert == replace {
            Ok(replace.clone())
        } else {
            let (old_val, old_key) = self.mapping.remove_entry(replace).unwrap();
            debug_assert!(old_key == key, "Keys for replace do not match");

            match self.mapping.insert(to_insert.clone(), key) {
                Some(_) => {
                    *self.mapping.get_mut(to_insert).unwrap() = old_key;
                    Err(UniqueConflict)
                }
                None => Ok(old_val),
            }
        }
    }

    // replace the old (successfully inserted) value (no copy required)
    pub fn undo_replace(&mut self, old_val: Field, update: &Field, key: Key) {
        self.mapping.remove(update).unwrap();
        let res = self.mapping.insert(old_val, key);
        debug_assert!(res.is_none(), "Undo replace failed");
    }
}
