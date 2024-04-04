use std::collections::LinkedList;

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