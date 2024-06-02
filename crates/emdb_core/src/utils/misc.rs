use std::collections::{HashMap, HashSet, LinkedList};

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

pub struct PushMap<'brw, K,V> {
    map: &'brw mut HashMap<K,V>,
}

impl <'brw, K,V> PushMap<'brw, K,V> where K: std::cmp::Eq + std::hash::Hash {
    pub fn new(map: &'brw mut HashMap<K,V>) -> Self {
        Self {
            map,
        }
    }

    pub fn push(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    
    pub fn len(&self) -> usize { self.map.len() }
}

pub struct PushSet<'brw,V> {
    map: &'brw mut HashSet<V>,
}

impl <'brw, V> PushSet<'brw, V> where V: std::cmp::Eq + std::hash::Hash {
    pub fn new(map: &'brw mut HashSet<V>) -> Self {
        Self {
            map,
        }
    }

    pub fn push(&mut self, value: V) {
        self.map.insert(value);
    }

    pub fn len(&self) -> usize { self.map.len() }
}