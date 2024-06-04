use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
};

pub(crate) fn singlelist<T>(item: T) -> LinkedList<T> {
    let mut list = LinkedList::new();
    list.push_back(item);
    list
}

pub(crate) fn result_to_opt<O, E>(res: Result<O, E>, errs: &mut LinkedList<E>) -> Option<O> {
    match res {
        Ok(o) => Some(o),
        Err(e) => {
            errs.push_back(e);
            None
        }
    }
}

pub struct PushMap<'brw, K, V> {
    map: &'brw mut HashMap<K, V>,
    push_cnt: usize,
}

impl<'brw, K: Hash + Eq, V> PushMap<'brw, K, V> {
    pub fn new(map: &'brw mut HashMap<K, V>) -> Self {
        Self { map, push_cnt: 0 }
    }

    pub fn push(&mut self, key: K, value: V) -> Option<V> {
        self.push_cnt += 1;
        self.map.insert(key, value)
    }

    pub fn count(&self) -> usize {
        self.push_cnt
    }
}

pub struct PushSet<'brw, K> {
    set: &'brw mut HashSet<K>,
    push_cnt: usize,
}

impl<'brw, K: Hash + Eq> PushSet<'brw, K> {
    pub fn new(set: &'brw mut HashSet<K>) -> Self {
        Self { set, push_cnt: 0 }
    }

    pub fn push(&mut self, key: K) -> bool {
        self.push_cnt += 1;
        self.set.insert(key)
    }

    pub fn count(&self) -> usize {
        self.push_cnt
    }
}
