//! Constraints applicable to individual columns, and to the entire table/rows.
//!
//! Each constraint can be optionally associated with an alias, used to generate
//! the error variant returned to the user when attempts to violate constraints
//! are reported.  

use proc_macro2::Ident;
use std::collections::HashMap;
use syn::Expr;

use super::{Key, Plan, ScalarType};

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
pub struct ColSelect {
    pub col: Ident,
    pub select_as: Ident,
}

impl Plan {
    pub fn get_table(&self, k: Key<Table>) -> &Table {
        self.tables.get(k).unwrap()
    }
}
