//! Track borrows over a dataflow graph, with some ordering.
//! - Needs to be accessed in a dataflow ordering over the graph.
//! - Produces the borrows and collects required.
//! 
//! By removing intermediate collections, we can improve performance for some operator implementations:
//! - Lazy implementations do not need to pull.
use std::collections::{HashMap, HashSet};

use crate::plan::{self, Collect};

enum BorrowKind {
    Read,
    Write,
}

type Borrows<'imm> = HashMap<plan::ImmKey<'imm, plan::Table>, BorrowKind>;

pub struct BorrowTracker<'imm> {
    previous: Borrows<'imm>,
    current: HashMap<plan::ImmKey<'imm, plan::DataFlow>, Borrows<'imm>>,
}

impl<'imm> BorrowTracker<'imm> {
    fn remove_dataflow(&mut self, key: plan::ImmKey<'imm, plan::DataFlow>) -> Borrows<'imm> {
        todo!()
    }

    fn add_dataflow(&mut self, key: plan::ImmKey<'imm, plan::DataFlow>) {
        todo!()
    }

    /// Add a new borrow, and return the collection that needs to occur for it to be valid
    fn add_brw(&mut self, key: plan::ImmKey<'imm, plan::DataFlow>, table: plan::ImmKey<'imm, plan::Table>, kind: BorrowKind) -> Collections<'imm> {
        todo!()
    }

    fn move_dataflow(
        &mut self,
        from: plan::ImmKey<'imm, plan::DataFlow>,
        to: plan::ImmKey<'imm, plan::DataFlow>,
    ) {
        todo!()
    }

    fn end_context(self) -> Borrows<'imm> {
        todo!()
    }
}

type Collections<'imm> = HashSet<plan::ImmKey<'imm, plan::DataFlow>>;

#[enumtrait::store(trait_borrow_tracked)]
trait BorrowTracked<'imm> {
    fn track_borrow(
        &self,
        lp: &'imm plan::Plan,
        tracker: &mut BorrowTracker<'imm>,
    ) -> Collections<'imm> {
        todo!()
    }
}

#[enumtrait::impl_trait(trait_borrow_tracked for plan::operator_enum)]
impl<'imm> BorrowTracked<'imm> for plan::Operator {}

impl<'imm> BorrowTracked<'imm> for plan::UniqueRef {
    fn track_borrow(
        &self,
        lp: &'imm plan::Plan,
        tracker: &mut BorrowTracker<'imm>,
    ) -> Collections<'imm> {
        let output = plan::ImmKey::new(self.output, lp);
        let input = plan::ImmKey::new(self.input, lp);
        let collects = tracker.add_brw(input, plan::ImmKey::new(self.table, lp), BorrowKind::Read);
        tracker.move_dataflow(input, output);
        collects
    }
}

impl<'imm> BorrowTracked<'imm> for plan::ScanRefs {}
impl<'imm> BorrowTracked<'imm> for plan::DeRef {}
impl<'imm> BorrowTracked<'imm> for plan::Update {}
impl<'imm> BorrowTracked<'imm> for plan::Insert {}
impl<'imm> BorrowTracked<'imm> for plan::Delete {}
impl<'imm> BorrowTracked<'imm> for plan::Map {}
impl<'imm> BorrowTracked<'imm> for plan::Expand {}
impl<'imm> BorrowTracked<'imm> for plan::Fold {}
impl<'imm> BorrowTracked<'imm> for plan::Filter {}
impl<'imm> BorrowTracked<'imm> for plan::Sort {}
impl<'imm> BorrowTracked<'imm> for plan::Assert {}
impl<'imm> BorrowTracked<'imm> for plan::Combine {}
impl<'imm> BorrowTracked<'imm> for plan::Count {}
impl<'imm> BorrowTracked<'imm> for plan::Take {}
impl<'imm> BorrowTracked<'imm> for plan::Collect {}
impl<'imm> BorrowTracked<'imm> for plan::Join {}
impl<'imm> BorrowTracked<'imm> for plan::Fork {}
impl<'imm> BorrowTracked<'imm> for plan::Union {}
impl<'imm> BorrowTracked<'imm> for plan::Row {}
impl<'imm> BorrowTracked<'imm> for plan::Return {}
impl<'imm> BorrowTracked<'imm> for plan::Discard {}

impl<'imm> BorrowTracked<'imm> for plan::GroupBy {}
impl<'imm> BorrowTracked<'imm> for plan::Lift {}
