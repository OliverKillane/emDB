use std::collections::{HashMap, HashSet};

use quote::quote;
use quote_debug::Tokens;
use syn::{Expr, ExprLet, Ident, Path, Type};

use super::{namer::SimpleNamer, tables::GeneratedInfo};

#[enumtrait::store(trait_operator_gen)]
pub trait OperatorGen {
    /// Generate the data needed that captures from query parameters
    fn closure_data<'imm>(
        &self,
        lp: &'imm plan::Plan,
        get_types: &HashMap<plan::Idx<'imm, plan::Table>, HashMap<Ident, Tokens<Type>>>,
        namer: &SimpleNamer,
    ) -> Tokens<Expr> {
        quote! { todo!() }.into()
    }

    /// Generate the code for the operator
    /// - Needs to update the set of mutated tables
    /// - Adds to the available errors
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        errors: &mut HashMap<Ident, Tokens<Path>>,
        mutated_tables: &mut HashSet<plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
    ) -> (Tokens<ExprLet>, bool) {
        let name = namer.operator_return_value_name(self_key);
        (quote! { let #name = todo!() }.into(), false)
    }
}

use crate::plan::{self, operator_enum};

#[enumtrait::impl_trait(trait_operator_gen for operator_enum)]
impl OperatorGen for plan::Operator {}

impl OperatorGen for plan::UniqueRef {
    fn apply<'imm>(
        &self,
        self_key: plan::Key<plan::Operator>,
        lp: &'imm plan::Plan,
        namer: &SimpleNamer,
        errors: &mut HashMap<Ident, Tokens<Path>>,
        mutated_tables: &mut HashSet<plan::ImmKey<'imm, plan::Table>>,
        gen_info: &GeneratedInfo<'imm>,
    ) -> (Tokens<ExprLet>, bool) {
        
        let data_in = lp.get_dataflow(self.input).get_conn();

        let data_in_name = namer.dataflow_error(self.input);
        let data_out_name = namer.dataflow_error(self_key);

        let code = if data_in.with.stream {
            quote!{

            }
        } else {
            quote!{
                let out_name: result_type = {
                    let field = self.table_name.unique_gen(&input.field);
                    let InData {} = input;
                    OutData{}
                };
            }
        }.into();

        (code, false)
    }
}

impl OperatorGen for plan::ScanRefs {}
impl OperatorGen for plan::DeRef {}
impl OperatorGen for plan::Update {}
impl OperatorGen for plan::Insert {}
impl OperatorGen for plan::Delete {}
impl OperatorGen for plan::Map {}
impl OperatorGen for plan::Expand {}
impl OperatorGen for plan::Fold {}
impl OperatorGen for plan::Filter {}
impl OperatorGen for plan::Sort {}
impl OperatorGen for plan::Assert {}
impl OperatorGen for plan::Take {}
impl OperatorGen for plan::Collect {}
impl OperatorGen for plan::GroupBy {}
impl OperatorGen for plan::ForEach {}
impl OperatorGen for plan::Join {}
impl OperatorGen for plan::Fork {}
impl OperatorGen for plan::Union {}
impl OperatorGen for plan::Row {}
impl OperatorGen for plan::Return {}
impl OperatorGen for plan::Discard {}
