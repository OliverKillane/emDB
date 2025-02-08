//! # Interface Trait for Comparing Implementations
//! - Allows for any return types, any table key types.
//! - Assumes a window-like (`datastore`, `database` wraps `&mut datastore`) pattern.

use crate::{analysis::mutability::GetMuts, plan, utils::on_off::on_off};
use combi::{
    tokens::{
        basic::{collectuntil, peekpunct, recovgroup},
        derived::listseptrailing,
        options::{OptEnd, OptField, OptParse},
        TokenDiagnostic, TokenIter, TokenParser,
    },
    Combi,
};
use namer::InterfaceNamer;
use proc_macro2::{Delimiter, TokenStream};
use quote::{quote, ToTokens};
use quote_debug::Tokens;
use syn::{Ident, Type};

pub struct Interface {
    public: bool,
    traits_for_db: Vec<TokenStream>,
    traits_with_db: Vec<TokenStream>,
}

impl super::EMDBBackend for Interface {
    const NAME: &'static str = "Interface";

    fn parse_options(
        backend_name: &Ident,
        options: Option<proc_macro2::TokenStream>,
    ) -> Result<Self, std::collections::LinkedList<proc_macro_error2::Diagnostic>> {
        fn get_traits() -> impl TokenParser<Vec<TokenStream>> {
            recovgroup(
                Delimiter::Brace,
                listseptrailing(',', collectuntil(peekpunct(','))),
            )
        }

        if let Some(opts) = options {
            let parser = (
                OptField::new("pub", on_off),
                (
                    OptField::new("traits_for_db", get_traits),
                    (OptField::new("traits_with_db", get_traits), OptEnd),
                ),
            )
                .gen('=');

            let (_, res) = parser.comp(TokenIter::from(opts, backend_name.span()));
            res.to_result().map_err(TokenDiagnostic::into_list).map(
                |(public, (traits_for_db, (traits_with_db, ())))| Interface {
                    public: public.unwrap_or(false),
                    traits_for_db: traits_for_db.unwrap_or(Vec::new()),
                    traits_with_db: traits_with_db.unwrap_or(Vec::new()),
                },
            )
        } else {
            Ok(Self {
                public: false,
                traits_for_db: Vec::new(),
                traits_with_db: Vec::new(),
            })
        }
    }

    fn generate_code(
        self,
        impl_name: Ident,
        plan: &plan::Plan,
    ) -> Result<proc_macro2::TokenStream, std::collections::LinkedList<proc_macro_error2::Diagnostic>>
    {
        let interface_namer = InterfaceNamer::new();
        let InterfaceNamer {
            trait_database,
            trait_database_type_datastore,
            trait_datastore,
            trait_datastore_type_database,
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

        let exposed_table_keys = public::exposed_keys(plan);
        let key_types = exposed_table_keys.into_iter().map(|tablekey| {
            let key_name = interface_namer.key_name(&plan.get_table(*tablekey).name);
            quote! { type #key_name: Clone + Copy + Eq }
        });

        let traits_for_db = if self.traits_for_db.is_empty() {
            quote!()
        } else {
            let trait_vec = &self.traits_for_db;
            quote!(: #(#trait_vec)+*)
        };

        let traits_with_db = if self.traits_with_db.is_empty() {
            quote!()
        } else {
            let trait_vec = &self.traits_with_db;
            quote!(#(+#trait_vec)*)
        };

        let public_tk = if self.public { quote!(pub) } else { quote!() };

        Ok(quote! {
            #public_tk mod #impl_name {
                #![allow(non_camel_case_types)]
                // NOTE: We want to allow methods to return any type, which would normally
                //       require the trait to have an associated type, and to use this in the
                //       return position of the method.
                //
                //       Or we can use the newer `impl Trait`, and then implement
                //       a trait for everything to get as close to `auto my_method(..)`
                //       as possible.
                pub trait #trait_any{}
                impl <T> #trait_any for T {}

                pub trait #trait_database<#db_lifetime> #traits_for_db {
                    type #trait_database_type_datastore: #trait_datastore;
                    #(#query_code)*
                }

                pub trait #trait_datastore {
                    // NOTE: the names of the datastore, and the database cannot conflict because the table names have `_key` appended.
                    type #trait_datastore_type_database<'imm>: #trait_database<'imm, #trait_database_type_datastore=Self> #traits_with_db where Self: 'imm;
                    #(#key_types;)*
                    fn #trait_datastore_method_new() -> Self;
                    fn #trait_datastore_method_db(&mut self) -> Self::#trait_datastore_type_database<'_>;
                }
            }
        })
    }
}

pub mod namer;
pub mod public; 

fn generate_parameter_type(
    lp: &plan::Plan,
    key: plan::Key<plan::ScalarType>,
    namer: &InterfaceNamer,
) -> Tokens<Type> {
    let InterfaceNamer {
        trait_any,
        trait_database_type_datastore,
        trait_datastore,
        ..
    } = namer;

    match lp.get_scalar_type_conc(key) {
        plan::ScalarTypeConc::TableRef(table_id) => {
            let key_name = namer.key_name(&lp.get_table(*table_id).name);
            quote! (<Self::#trait_database_type_datastore as #trait_datastore>::#key_name)
        }
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
