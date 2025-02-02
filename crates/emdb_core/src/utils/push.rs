//! ## Push only wrappers for [`HashSet`] and [`HashMap`]
//! allows the tracking of if the structures are mutated in different scopes,
//! while allowing all inserts to be sent to the same `&mut` data structure.  

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
};

pub trait PushMap<K, V> {
    fn scope(&mut self) -> impl PushMap<K, V> + '_;
    fn push(&mut self, key: K, value: V) -> Option<V>;
    fn pushed(&self) -> bool;
}

pub struct PushMapConc<K, V> {
    map: HashMap<K, V>,
    pushed: bool,
}

impl<K, V> PushMapConc<K, V> {
    pub fn new(map: HashMap<K, V>) -> Self {
        Self { map, pushed: false }
    }

    pub fn extract(self) -> HashMap<K, V> {
        self.map
    }
}

struct RefMap<'brw, K, V, PM: PushMap<K, V>> {
    map: &'brw mut PM,
    pushed: bool,
    phantom: PhantomData<(K, V)>,
}

impl<K: Hash + Eq, V> PushMap<K, V> for PushMapConc<K, V> {
    fn scope(&mut self) -> impl PushMap<K, V> + '_ {
        RefMap {
            map: self,
            pushed: false,
            phantom: PhantomData,
        }
    }

    fn push(&mut self, key: K, value: V) -> Option<V> {
        self.pushed = true;
        self.map.insert(key, value)
    }

    fn pushed(&self) -> bool {
        self.pushed
    }
}

impl<K: Hash + Eq, V, PM: PushMap<K, V>> PushMap<K, V> for RefMap<'_, K, V, PM> {
    fn scope(&mut self) -> impl PushMap<K, V> + '_ {
        RefMap {
            map: self,
            pushed: false,
            phantom: PhantomData,
        }
    }

    fn push(&mut self, key: K, value: V) -> Option<V> {
        self.pushed = true;
        self.map.push(key, value)
    }

    fn pushed(&self) -> bool {
        self.pushed
    }
}

pub trait PushSet<K> {
    fn scope(&mut self) -> impl PushSet<K> + '_;
    fn push(&mut self, key: K) -> bool;
    fn pushed(&self) -> bool;
}

pub struct PushSetConc<K> {
    set: HashSet<K>,
    pushed: bool,
}

impl<K> PushSetConc<K> {
    pub fn new(set: HashSet<K>) -> Self {
        Self { set, pushed: false }
    }

    pub fn extract(self) -> HashSet<K> {
        self.set
    }
}

struct RefSet<'brw, K, PS: PushSet<K>> {
    set: &'brw mut PS,
    pushed: bool,
    phantom: PhantomData<K>,
}

impl<K: Hash + Eq> PushSet<K> for PushSetConc<K> {
    fn scope(&mut self) -> impl PushSet<K> + '_ {
        RefSet {
            set: self,
            pushed: false,
            phantom: PhantomData,
        }
    }

    fn push(&mut self, key: K) -> bool {
        self.pushed = true;
        self.set.insert(key)
    }

    fn pushed(&self) -> bool {
        self.pushed
    }
}

impl<K: Hash + Eq, PS: PushSet<K>> PushSet<K> for RefSet<'_, K, PS> {
    fn scope(&mut self) -> impl PushSet<K> + '_ {
        RefSet {
            set: self,
            pushed: false,
            phantom: PhantomData,
        }
    }

    fn push(&mut self, key: K) -> bool {
        self.pushed = true;
        self.set.push(key)
    }

    fn pushed(&self) -> bool {
        self.pushed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_pushmap() {
        let mut push_map = PushMapConc::new(HashMap::new());
        assert!(!push_map.pushed());
        push_map.push(12, "hello");
        assert!(push_map.pushed());
        {
            let mut push_map_1 = push_map.scope();
            assert!(!push_map_1.pushed());
            {
                let mut push_map_2 = push_map_1.scope();
                assert!(!push_map_2.pushed());
                push_map_2.push(13, "world");
                assert!(push_map_2.pushed());
            };
            assert!(push_map_1.pushed());
            {
                let mut push_map_2 = push_map_1.scope();
                assert!(!push_map_2.pushed());
                push_map_2.push(13, "world");
                assert!(push_map_2.pushed());
            };
        }
    }
}
