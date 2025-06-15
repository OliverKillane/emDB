use crate::plan::ir;

pub mod names;
pub mod recur;
pub mod params;

pub trait Semantic<'consts, 'datas, 'stages, N: ir::namer::Namer> {
    type Error;

    /// JUSTIFY: Single semantic error
    ///  - Multiple semantic errors are necessary when the user needs fast feedback
    ///    on editing input frequently
    ///  - serbit is intended for layout/protocol generation, so the user will
    ///    run/change the input rarely.
    ///  Hence we go without, and can simplify the code as a result.
    fn analyse(plan: &ir::Plan<'consts, 'datas, 'stages, N>) -> Result<(), Self::Error>;
}
