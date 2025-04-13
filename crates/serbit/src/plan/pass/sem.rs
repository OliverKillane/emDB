//! ## Sem
//! A sanity check on the plan.
//!  - Frontends should ensure they conform to the semantics of [ir], this pass checks that the
//!    [ir::Plan] generated is correct.

use crate::{
    plan::ir::{self, item_enum, item_stage},
    utils::scope::{Scope, ScopeRoot},
};
use smart_arenas::{arena::Arena, key::KeyTrait};
use std::collections::HashSet;

struct SemError;

fn analyse(plan: &ir::Plan<impl ir::Naming>) -> Result<(), SemError> {
    todo!()
}

trait ValidateExpr {
    fn validate<'plan>(&'plan self, scope: &Scope<&'plan ir::keys::Item, ()>);
}

impl ValidateExpr for ir::Int {
    fn validate<'plan>(&'plan self, scope: &Scope<&'plan ir::keys::Item, ()>) {
        match self {
            ir::Int::Const { value, kind } => todo!(),
            ir::Int::Bin(math_bin_op, key, key1) => todo!(),
            ir::Int::Ref(key) => todo!(),
            ir::Int::Choice(key, key1, key2) => todo!(),
        }
    }
}

impl ValidateExpr for ir::Bool {
    fn validate<'plan>(&'plan self, scope: &Scope<&'plan ir::keys::Item, ()>) {
        match self {
            ir::Bool::Const(_) => todo!(),
            ir::Bool::Not(key) => todo!(),
            ir::Bool::Logic(logical_bin_op, key, key1) => todo!(),
            ir::Bool::Arith(arith_bin_op, key, key1) => todo!(),
        }
    }
}

#[enumtrait::store(validate_bool)]
trait ValidateBool {
    fn validate<'plan>(&'plan self, scope: &Scope<&'plan ir::keys::Item, ()>);
}

#[enumtrait::store(validate_item)]
trait ValidateItem {
    fn validate<'plan>(
        &'plan self,
        plan: &'plan ir::Plan<impl ir::Naming>,
        scope: &Scope<&'plan ir::keys::Item, ()>,
        current_stage: &mut HashSet<&'plan ir::keys::Item>,
    );
}

#[enumtrait::impl_trait(validate_item for item_enum)]
impl ValidateItem for ir::Item {}

enum ItemError<'plan, N: ir::Naming> {
    DuplicateItem(&'plan ir::keys::Item),
    DuplicateName {
        name: &'plan N::Ident,
        other: &'plan N::Ident,
    },
}

impl ValidateItem for ir::Array {
    fn validate<'plan>(
        &'plan self,
        plan: &'plan ir::Plan<impl ir::Naming>,
        scope: &Scope<&'plan ir::keys::Item, ()>,
        current_stage: &mut HashSet<&'plan ir::keys::Item>,
    ) {
        current_stage.insert(&self.item);
    }
}
impl ValidateItem for ir::Choice {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: &Scope<&'a ir::keys::Item, ()>,
        current_stage: &mut HashSet<&'a ir::keys::Item>,
    ) {
        todo!()
    }
}
impl ValidateItem for ir::Tuple {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: &Scope<&'a ir::keys::Item, ()>,
        current_stage: &mut HashSet<&'a ir::keys::Item>,
    ) {
        todo!()
    }
}
impl ValidateItem for ir::Primitive {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: &Scope<&'a ir::keys::Item, ()>,
        current_stage: &mut HashSet<&'a ir::keys::Item>,
    ) {
        todo!()
    }
}

#[enumtrait::store(validate_stage)]
trait ValidateStage {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: Scope<&'a ir::keys::Item, ()>,
    );
}

#[enumtrait::impl_trait(validate_stage for item_stage)]
impl ValidateStage for ir::Stage {}

impl ValidateStage for ir::Single {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: Scope<&'a ir::keys::Item, ()>,
    ) {
        todo!()
    }
}
impl ValidateStage for ir::Repeat {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: Scope<&'a ir::keys::Item, ()>,
    ) {
        todo!()
    }
}
impl ValidateStage for ir::Until {
    fn validate<'a>(
        &self,
        plan: &'a ir::Plan<impl ir::Naming>,
        scope: Scope<&'a ir::keys::Item, ()>,
    ) {
        todo!()
    }
}
