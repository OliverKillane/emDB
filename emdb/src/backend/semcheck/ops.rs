//! Generate the closures provided by the user for use in queries.
//! 
//! All operator's expressions can access the query parameters, capturing 
//! immutable references, or moving the values.
//! 
//! Hence we need to build the expressions with the following constraints:
//! 1. the order for capture is the same as their occurences in the query
//!    - so the user can recieve meaningful error messages captures, moves, etc.
//!    - to allow further captures from contexts (e.g. a group-by's grouping 
//!      parameter)
//! 2. The capture needs to be truly zero cost, no boxed closures, with the 
//!    closures as trivially inlinable functions
//!
//! Here we are constrained by rust's current type system / language limitations
//! - the anonymous types (used by by closures) is not describable by the user 
//!   so return position `impl {trait}` must be used.
//! - the return type of some closures, is another closure (for example groupby 
//!   produces a closure containing types)
//! - for a flat query with a single context per query, we can avoid this by 
//!   using a single level of closures
//!
//! To get around this, we avoid describing the type at all costs.
//! - the types of nested lambdas are entirely inferred.
//! 
//! If more complex types are needed (groupby needs to describe type taken in), then we can 
//! 
//! 





// use crate::plan;

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

// fn translate_context(
//     lp: &plan::Plan, 
//     plan::Context { 
//         ordering, 
//         params, 
//         returnflow, 
//         discards 
//     }: &plan::Context,
//     types@TransTypes { records, scalars }: &TransTypes
// ) -> RustExpression {
//     let trans_params = params.iter().map(|(id, scalar_key)| {
//         let t = scalars.get(&scalar_key.to_idx()).unwrap();
//         quote! { #id: #t }
//     });
    
//     let (trans_ops, trans_types): (Vec<_>, Vec<_>) = ordering.iter().map(|op_key| {
//         let rs_expr = lp.get_operator(*op_key).generate_context_exprs(lp, *op_key, types);
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

// fn scalar_typename(key: plan::Key<plan::ScalarType>) -> Ident {
//     Ident::new(&format!("ScalarType_{}", key.to_idx()), Span::call_site())
// }
// fn record_typename(key: plan::Key<plan::Record>) -> Ident {
//     Ident::new(&format!("RecordType_{}", key.to_idx()), Span::call_site())
// }

// #[enumtrait::store(trait_context_trans)]
// trait ContextTrans: Sized {
//     fn generate_context_exprs(&self, lp: &plan::Plan, self_key: plan::Key<plan::Operator>) -> RustExpression;
// }


// // #[enumtrait::impl_trait(trait_context_trans for plan::operator_enum)]
// impl ContextTrans for plan::Operator {
//     fn generate_context_exprs(&self,lp: &plan::Plan,self_key:plan::Key<plan::Operator>) -> RustExpression {
//         todo!()
//     }
// }

// // Generate a closure, providing access to the names of the fields in the dataflow
// fn get_dataflow_closure(lp: &plan::Plan, df_key: plan::Key<plan::DataFlow>, RustExpression{ expression, data_type }: RustExpression) -> RustExpression {
//     let plan::DataFlowConn { from, to, with } = lp.get_dataflow(df_key).get_conn();
//     let rec_name = record_typename(with.fields);
//     let names = lp.get_record_type(with.fields).fields.iter().map(|(name, _)| name);
//     let closure_type = if with.stream {quote!(Fn)} else {quote!(FnOnce)};
//     RustExpression::from(quote! {
//         | move #rec_name { #(#names,)* } | {
//             #expression
//         }
//     }, quote!(impl #closure_type(#rec_name) -> ( #data_type )))
// }

// impl ContextTrans for plan::Update {
//     // TODO: has to read all table fields unecessarily
//     // TODO: shadowing between table and dataflow fields
//     // TODO: investigate lambda inlining / const prop
//     fn generate_context_exprs(&self, lp: &plan::Plan, self_key: plan::Key<plan::Operator>) -> RustExpression {        
//         // generate a function that takes in the table parameters, and outputs 
//         // dataflow, then the table, and outputs the updated table values
        
//         // let table_params = lp.get_table(self.table);

//         let table_fields = 


//         RustExpression::from(
//             quote! {
//                 |/* table params */| {
                
//                 }
//             },
//         )
//     }
// }
// // impl ContextTrans for plan::Insert {}
// // impl ContextTrans for plan::Delete {}
// // impl ContextTrans for plan::GetUnique {}
// // impl ContextTrans for plan::Scan {}
// // impl ContextTrans for plan::DeRef {}
// // impl ContextTrans for plan::Map {}
// // impl ContextTrans for plan::Fold {}
// // impl ContextTrans for plan::Filter {}
// // impl ContextTrans for plan::Sort {}
// // impl ContextTrans for plan::Assert {}
// // impl ContextTrans for plan::Collect {}
// // impl ContextTrans for plan::Take {}
// // impl ContextTrans for plan::Join {}
// // impl ContextTrans for plan::GroupBy {}
// // impl ContextTrans for plan::Fork {}
// // impl ContextTrans for plan::Union {}
// // impl ContextTrans for plan::Row {}
// // impl ContextTrans for plan::Return {}
// // impl ContextTrans for plan::Discard {}
