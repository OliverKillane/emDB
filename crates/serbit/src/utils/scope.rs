use std::{collections::HashMap, hash::Hash};

pub struct ScopeRoot<K: Eq + Hash, V> {
    data: Vec<HashMap<K, V>>,
}

impl<K: Eq + Hash, V> ScopeRoot<K, V> {
    pub fn new() -> Self {
        ScopeRoot {
            data: vec![HashMap::new()],
        }
    }

    pub fn first(&mut self) -> Scope<'_, K, V> {
        self.data.push(HashMap::new());
        Scope { root: self }
    }
}

pub struct Scope<'brw, K: Eq + Hash, V> {
    root: &'brw mut ScopeRoot<K, V>,
}

impl<'brw, K: Eq + Hash, V> Scope<'brw, K, V> {
    pub fn child(&mut self) -> Scope<'_, K, V> {
        self.root.data.push(HashMap::new());
        Scope { root: self.root }
    }

    pub fn add(&mut self, key: K, value: V) -> Option<V> {
        self.root.data.last_mut().unwrap().insert(key, value)
    }

    pub fn check(&mut self, key: &K) -> Option<&V> {
        for map in self.root.data.iter().rev() {
            if let Some(value) = map.get(key) {
                return Some(value);
            }
        }
        None
    }
}
impl<'brw, K: Eq + Hash, V> Drop for Scope<'brw, K, V> {
    fn drop(&mut self) {
        self.root.data.pop().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut root = ScopeRoot::new();
        let mut scope1 = root.first();
        scope1.add(1, ());

        {
            let mut scope2 = scope1.child();
            scope2.add(2, ());

            assert!(scope2.check(&1).is_some());
            assert!(scope2.check(&2).is_some());
        }

        assert!(scope1.check(&1).is_some());
        assert!(scope1.check(&2).is_none());
    }
}
