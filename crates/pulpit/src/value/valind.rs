//! # Indexed values for Unstable Columns
//! TODO: write description
//! TODO: explain issues with deref, negative trait bounds, and 
//! 

use crate::column;
use super::ImmutVal;



struct ValInd<Val> {
    data: Val
}

struct IndWrap<'db, StoreMut, Store, SubVal, Col, Select>
where 
    Col: column::Column<'db, StoreMut, Store, ImmVal = ValInd<Store>>,
    ValInd<Store> : ImmutVal<'db, Store>,
    // NOTE: In order for the `for<'a> Fn(..) -> &'a ..` to work, we need to use the lifetime 
    //       for a reference in the `Fn`'s input. Hence `&'a Col` is just a convenient dummy here
    Select: for<'a> Fn(<ValInd<Store> as ImmutVal<'db, Store>>::Brw<'a>, &'a Col) -> &'a SubVal
{
    ind: Col::Ind,
    select: Select,
}

impl <'db, StoreMut, Store, SubVal, Col, Select> IndWrap<'db, StoreMut, Store, SubVal, Col, Select>
where 
    Col: column::Column<'db, StoreMut, Store, ImmVal = ValInd<Store>>,
    ValInd<Store> : ImmutVal<'db, Store>,
    Select: for<'a>  Fn(<ValInd<Store> as ImmutVal<'db, Store>>::Brw<'a>, &'a Col) -> &'a SubVal
{
    /// Take a reference to the column (prevent relocation) borrow using the 
    /// indexing ref.
    fn deref<'a>(&'a self, col: &'a Col) -> &'a SubVal {
        let (imm, _) = col.brw(self.ind);
        (self.select)(imm, col)
    }
}
