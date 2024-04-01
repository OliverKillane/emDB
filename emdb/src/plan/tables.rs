//! Constraints applicable to individual columns, and to the entire table/rows.
//!
//! Each constraint can be optionally associated with an alias, used to generate
//! the error variant returned to the user when attempts to violate constraints
//! are reported.  

use proc_macro2::Ident;
use std::collections::HashMap;
use syn::Expr;

use super::{ConcRef, Key, LogicalPlan, Record, RecordConc, ScalarType, ScalarTypeConc};

pub struct Constraint<C> {
    pub alias: Option<Ident>,
    pub cons: C,
}

pub struct Unique;
pub struct Limit(pub Expr);
pub struct Pred(pub Expr);

pub struct ColumnConstraints {
    pub unique: Option<Constraint<Unique>>,
}
pub struct RowConstraints {
    pub limit: Option<Constraint<Limit>>,
    pub preds: Vec<Constraint<Pred>>,
}

pub struct Column {
    pub cons: ColumnConstraints,
    pub data_type: Key<ScalarType>,
}

pub struct Table {
    pub name: Ident,
    pub row_cons: RowConstraints,
    pub columns: HashMap<Ident, Column>,
}

#[derive(Clone)]
pub enum TableAccess {
    Ref(Ident),
    AllCols,
    Selection(Vec<Ident>),
}

pub fn generate_access(
    table_id: Key<Table>,
    access: TableAccess,
    lp: &mut LogicalPlan,
) -> Result<Key<Record>, Vec<Ident>> {
    match access {
        TableAccess::Ref(id) => {
            let ref_id = lp
                .scalar_types
                .insert(ConcRef::Conc(ScalarTypeConc::TableRef(table_id)));
            Ok(lp.record_types.insert(ConcRef::Conc(RecordConc {
                fields: HashMap::from([(id, ref_id)]),
            })))
        }
        TableAccess::AllCols => {
            let table = lp.get_table(table_id);
            Ok(lp.record_types.insert(ConcRef::Conc(RecordConc {
                fields: table
                    .columns
                    .iter()
                    .map(|(id, Column { cons, data_type })| (id.clone(), data_type.clone()))
                    .collect(),
            })))
        }
        TableAccess::Selection(ids) => {
            let table = lp.get_table(table_id);
            let mut fields = HashMap::new();
            let mut invalid_access = Vec::new();
            for id in ids {
                if let Some(Column { cons, data_type }) = table.columns.get(&id) {
                    fields.insert(id, *data_type);
                } else {
                    invalid_access.push(id);
                }
            }

            if invalid_access.is_empty() {
                Ok(lp.record_types.insert(ConcRef::Conc(RecordConc { fields })))
            } else {
                Err(invalid_access)
            }
        }
    }
}

impl LogicalPlan {
    pub fn get_table(&self, k: Key<Table>) -> &Table {
        self.tables.get(k).unwrap()
    }
}
