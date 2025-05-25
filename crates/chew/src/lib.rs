#![feature(generic_const_exprs)]

pub mod access;
pub mod cursor;
pub mod truth;
pub mod gen2;

#[cfg(test)]
mod tests {
    use crate::access::*;
    use crate::cursor::*;

    fn foo<C: ReadCursor<Bound = BoundConst<64>>>(cursor: &C, access: &AccessConst<1, 62, 0>) {
        cursor.read(access);
    }
}
