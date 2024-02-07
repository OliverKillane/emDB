use std::collections::LinkedList;

pub(crate) fn singlelist<T>(item: T) -> LinkedList<T> {
    let mut list = LinkedList::new();
    list.push_back(item);
    list
}
