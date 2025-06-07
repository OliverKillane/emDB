use crate::plan::ir;

pub mod names;
pub mod order;
pub mod recur;

pub trait Semantic<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: ir::Namer> {
    type Error;

    // JUSTIFY: Single semantic error
    //  - Multiple semantic errors are necessary when the user needs fast feedback 
    //    on editing input frequently 
    //  - serbit is intended for layout/protocol generation, so the user will 
    //    run/change the input rarely.
    //  Hence we go without, and can simplify the code as a result.
    fn analyse(
        plan: &ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>,
    ) -> Result<(), Self::Error>;
}
