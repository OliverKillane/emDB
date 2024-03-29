//! Constraints applicable to individual columns, and to the entire table/rows.
//! 
//! Each constraint can be optionally associated with an alias, used to generate 
//! the error variant returned to the user when attempts to violate constraints 
//! are reported.  
 
use std::collections::HashMap;
use proc_macro2::Ident;
use syn::Expr;

use super::types::ScalarType;

pub struct Constraint<C> {
    alias: Option<Ident>,
    cons: C
}

pub struct Unique;
pub struct Limit(Expr);
pub struct Pred(Expr);

pub struct ColumnConstraints {
    pub unique: Option<Constraint<Unique>>,
}
pub struct RowConstraints {
    pub limit: Option<Constraint<Limit>>,
    pub preds: Vec<Constraint<Pred>>,
}

pub struct Column {
    pub cons: ColumnConstraints,
    pub data_type: ScalarType,
}

pub struct Table {
    pub name: Ident,
    pub row_cons: RowConstraints,
    pub columns: HashMap<Ident, Column>,
}