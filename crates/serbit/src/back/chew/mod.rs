use super::*;

/// A backend generating code for use with the chew library.
///  - Support for bit-aligned types
///  - Access through methods, rather than casted structs
pub struct Chew;

impl <'consts, 'datas, 'stages, N: ir::namer::Namer> Backend<'consts, 'datas, 'stages, N> for Chew {
    type Error = ();

    fn generate(plan: &ir::Plan<'consts, 'datas, 'stages, N>) -> Result<Tokens<ItemMod>, Self::Error> {
        // generate the constants
        // generate the type structs
        // generate stages
        // generate the messages
        unimplemented!()

    }
}
/*

stages {
    stage aaa {
        struct foo {
            f1: [u8; 4],
            f2: u16,
            f4: struct bing {}
        }
    },
    stage bbb {
        struct bar {
            f3: [u8; super.foo.f2]     
        }
    },
}

*/

/*
mod protocol {
    mod metadata {
        struct Type0 {
            f1: Type1,
            f2: Type2,
        }

        struct Type1 {
            ctx_0: u16,
            item: Type2,
        }
        
        enum Type2<> {
            Item1(TypeN..),
            Item2(TypeM..),
            Item3(TypeO..),
        }
    }

    struct Reader<C: Cursor, M> {
        cursor: C,
        metadata: M,
    }

    impl Reader<C: Cursor, metadata::Type0> {
        fn f1(&self) -> Reader<C, metadata::Type1> { unimplemented!() }
        fn f2(&self) -> Reader<C, metadata::Type2> { unimplemented!() }
    }

    impl Reader<C: Cursor, metadata::Type1> {
        fn len(&self) -> usize { unimplemented!() }
        fn get(&self, idx: usize) -> { unimplemented!() }
        fn iter() -> impl Iterator<Item=Reader<C, metadata::Type2>> { unimplemented!() }
    }

    impl Reader<C: Cursor, metadata::Type2> {
        fn get(&self) -> metadata::Type2 { 
            need to output a reader based on the type inside. Which is based on
        }
    }
}







*/