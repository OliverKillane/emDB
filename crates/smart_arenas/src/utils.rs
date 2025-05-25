use crate::arena::Arena;

pub struct KeyWith<'brw, 'id, A: Arena<'id>> {
    pub key: A::Key,
    pub arena: &'brw A,
}

impl<'brw, 'id, A: Arena<'id>> KeyWith<'brw, 'id, A> {
    pub fn new(key: A::Key, arena: &'brw A) -> Self {
        Self { key, arena }
    }
}
