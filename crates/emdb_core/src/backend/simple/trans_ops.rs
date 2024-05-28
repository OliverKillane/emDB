//! # Operation Translation
//! Converting operators to invocations of the [super::physical_ops]

use proc_macro2::TokenStream;

trait Translator {
    fn invoke(&self) -> TokenStream;
}

/*


struct DB {
    table0: table,
    table1: table,
    ...
}

struct Window {
    table0: tablewindow,
    table1: tablewindow,
}


impl Window {

    fn x(&self) -> Result<> {

        table0.insert(aaa);
    }
}

*/