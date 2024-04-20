// //! Generates closures for each [`plan::Context`] that can be used when building queries.
// use crate::plan;
// use proc_macro2::{TokenStream, Ident, Span};
// use syn::{parse2, Type};
// use quote::{quote, ToTokens};

// struct RustExpression {
//     expression: TokenStream,
//     data_type: Type,
// }

// impl RustExpression {
//     fn from(expression: TokenStream, data_type: TokenStream) -> Self {
//         Self {
//             expression,
//             data_type: parse2(data_type).expect("Invalid data type generated"),
//         }
//     }
// }

// fn scalar_typename(key: plan::Key<plan::ScalarType>) -> Ident {
//     Ident::new(&format!("ScalarType_{}", key.to_idx()), Span::call_site())
// }
// fn record_typename(key: plan::Key<plan::RecordType>) -> Ident {
//     Ident::new(&format!("RecordType_{}", key.to_idx()), Span::call_site())
// }
// fn record_fieldname(field: &plan::RecordField) -> Ident {
//     match field {
//         plan::RecordField::User(i) => i.clone(),
//         plan::RecordField::Internal(i) => Ident::new(&format!(""), Span::call_site()),
//     }
// }

// // Generate a closure, providing access to the names of the fields in the dataflow
// fn get_dataflow_closure(lp: &plan::Plan, df_key: plan::Key<plan::DataFlow>, RustExpression{ expression, data_type }: RustExpression) -> RustExpression {
//     let plan::DataFlowConn { from, to, with } = lp.get_dataflow(df_key).get_conn();
//     let rec_name = record_typename(with.fields);
//     let names = lp.get_record_type(with.fields).fields.iter().map(|(name, _)| record_fieldname(name));
//     let closure_type = if with.stream {quote!(Fn)} else {quote!(FnOnce)};
//     RustExpression::from(quote! {
//         | move #rec_name { #(#names,)* } | {
//             #expression
//         }
//     }, quote!(impl #closure_type(#rec_name) -> ( #data_type )))
// }

// fn translate_context(
//     lp: &plan::Plan,
//     plan::Context {
//         ordering,
//         params,
//         returnflow,
//         discards
//     }: &plan::Context,
// ) -> RustExpression {
//     let trans_params = params.iter().map(|(id, scalar_key)| {
//     let t = scalar_typename(*scalar_key);
//     quote! { #id: #t }
// });
//     let (trans_ops, trans_types): (Vec<_>, Vec<_>) = ordering.iter().map(|op_key| {
//         let rs_expr = lp.get_operator(*op_key).generate_context_exprs(lp, *op_key);
//         (rs_expr.expression, rs_expr.data_type)
//     }).unzip();

//     RustExpression::from(
//         quote! {
//             | #(#trans_params,)* | {
//                 (
//                     #(#trans_ops,)*
//                 )
//             }
//         },
//         quote!{ ( #(#trans_types,)* ) },
//     )
// }

// #[enumtrait::store(generate_context_exprs_trait)]
// trait GenerateContextExprs {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression;
// }

// #[enumtrait::impl_trait(generate_context_exprs_trait for plan::operator_enum)]
// impl GenerateContextExprs for plan::Operator {}

// impl GenerateContextExprs for plan::Update {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Insert {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Delete {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::GetUnique {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::ScanRefs {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::DeRef {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Map {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Expand {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Fold {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Filter {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Sort {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Assert {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Collect {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Take {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Join {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::GroupBy {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Fork {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Union {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Row {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Return {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
// impl GenerateContextExprs for plan::Discard {
//     fn generate_context_exprs(&self, lp: &plan::Plan) -> RustExpression {
//         todo!()
//     }
// }
