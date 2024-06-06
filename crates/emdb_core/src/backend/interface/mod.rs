//! # Interface Trait for Comparing Implementations
//! - Allows for any return types, any table key types.
//! - Assumes a window-like (`datastore`, `database` wraps `&mut datastore`) pattern.

use crate::{analysis::mutability::GetMuts, plan, utils::misc::singlelist};
use namer::InterfaceNamer;
use proc_macro_error::{Diagnostic, Level};
use quote::{quote, ToTokens};
use quote_debug::Tokens;
use syn::{spanned::Spanned, Ident, Type};

pub struct Interface;

impl super::EMDBBackend for Interface {
    const NAME: &'static str = "Interface";

    fn parse_options(
        backend_name: &Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error::Diagnostic>> {
        if let Some(opts) = options {
            if opts.is_empty() {
                Ok(Self)
            } else {
                Err(singlelist(Diagnostic::spanned(
                    opts.span(),
                    Level::Error,
                    format!("No options are taken for {}", Self::NAME),
                )))
            }
        } else {
            Ok(Self)
        }
    }

    fn generate_code(
        self,
        impl_name: Ident,
        plan: &plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error::Diagnostic>>
    {
        let interface_namer = InterfaceNamer::new();
        let InterfaceNamer {
            trait_database,
            trait_datastore,
            trait_datastore_method_new,
            trait_datastore_method_db,
            trait_any,
        } = &interface_namer;
        let db_lifetime = quote! {'db};
        let qy_lifetime = quote! {'qy};

        let query_code = plan
            .queries
            .iter()
            .map(|(_, plan::Query { name, ctx })| {
                let mut_tk = if plan.get_context(*ctx).mutates(plan) {
                    quote!(mut)
                } else {
                    quote!()
                };

                let params = plan.get_context(*ctx).params.iter().map(|(name, ty)| {
                    let ty = generate_parameter_type(plan, *ty, &interface_namer);
                    quote! { #name: #ty }
                });

                quote! {
                    fn #name<#qy_lifetime>(&#qy_lifetime #mut_tk self, #(#params),* ) -> impl #trait_any;
                }
            });

        Ok(quote! {
            mod #impl_name {
                // NOTE: We want to allow methods to return any type, which would normally
                //       require the trait to have an associated type, and to use this in the
                //       return position of the method.
                //
                //       Or we can use the newer `impl Trait`, and then implement
                //       a trait for everything to get as close to `auto my_method(..)`
                //       as possible.
                pub trait #trait_any{}
                impl <T> #trait_any for T {}

                pub trait #trait_database<#db_lifetime> {
                    #(#query_code)*
                }

                pub trait #trait_datastore {
                    fn #trait_datastore_method_new() -> Self;
                    fn #trait_datastore_method_db(&mut self) -> impl #trait_database<'_>;
                }
            }
        })
    }
}

pub mod namer;

fn generate_parameter_type(
    lp: &plan::Plan,
    key: plan::Key<plan::ScalarType>,
    namer: &InterfaceNamer,
) -> Tokens<Type> {
    let InterfaceNamer { trait_any, .. } = namer;

    match lp.get_scalar_type_conc(key) {
        plan::ScalarTypeConc::TableRef(_) => quote! (impl #trait_any),
        plan::ScalarTypeConc::Rust {
            type_context: plan::TypeContext::Query,
            ty,
        } => ty.to_token_stream(),
        _ => unreachable!("Only rust types and table references are allowed in query parameters"),
    }
    .into()
}

pub struct InterfaceTrait {
    pub name: Ident,
}