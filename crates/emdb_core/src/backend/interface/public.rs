
use std::collections::HashSet;

use crate::plan;
fn get_exposed_keys_record<'imm>(lp: &'imm plan::Plan, key: plan::Key<plan::RecordType>, tableset: &mut HashSet<plan::ImmKey<'imm, plan::Table>>) {
    for (_, k) in &lp.get_record_type_conc(key).fields {
        get_exposed_keys_scalar(lp, *k, tableset)
    }
}
fn get_exposed_keys_scalar<'imm>(lp: &'imm plan::Plan, key: plan::Key<plan::ScalarType>, tableset: &mut HashSet<plan::ImmKey<'imm, plan::Table>>) {
    match lp.get_scalar_type_conc(key) {
        plan::ScalarTypeConc::TableRef(t) => {tableset.insert(plan::ImmKey::new(*t, lp));},
        plan::ScalarTypeConc::Bag(r) | plan::ScalarTypeConc::Record(r) => get_exposed_keys_record(lp, *r, tableset),
        _ => (),
    }
}

pub fn exposed_keys<'imm>(lp: &'imm plan::Plan) -> HashSet<plan::ImmKey<'imm, plan::Table>> {
    let mut tableset = HashSet::new();
    for (_, plan::Query{ctx, ..}) in &lp.queries {
        let context = lp.get_context(*ctx);
        for (_, ty) in &context.params {
            get_exposed_keys_scalar(lp, *ty, &mut tableset)
        }
        if let Some(ty) = context.get_return_type(lp) {
            get_exposed_keys_record(lp, ty, &mut tableset)
        }
    }
    tableset
}
