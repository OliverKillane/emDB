use super::SingleOp;
use crate::{
    columns::PrimaryKind,
    groups::{FieldIndex, Groups},
    namer::CodeNamer,
};
use proc_macro2::TokenStream;
use quote::quote;

fn generate_borrow_fields<'a, Primary: PrimaryKind>(
    groups: &'a Groups<Primary>,
    namer: &'a CodeNamer,
) -> impl Iterator<Item = TokenStream> + 'a {
    groups.idents.iter().map(|(field_name, field_index)| {
        let data = match field_index {
            FieldIndex::Primary(_) => namer.name_primary_column.clone(),
            FieldIndex::Assoc {
                assoc_ind,
                inner: _,
            } => namer.name_assoc_column(*assoc_ind),
        };

        let imm_access = if field_index.is_imm() {
            quote!(imm_data)
        } else {
            quote!(mut_data)
        };

        quote!(#field_name: &#data.#imm_access.#field_name)
    })
}

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let CodeNamer {
        type_key,
        struct_window,
        pulpit_path,
        name_primary_column,
        type_key_error,
        table_member_columns,
        mod_borrow,
        mod_borrow_struct_borrow,
        method_borrow,
        ..
    } = namer;

    let borrowed_fields = generate_borrow_fields(groups, namer);
    let struct_fields = groups.idents.iter().map(|(field_name, field_index)| {
        let field_ty = groups.get_type(field_index).unwrap();
        quote!(pub #field_name: &'brw #field_ty)
    });

    let assoc_brws = (0..groups.assoc.len()).map(|ind| {
        let name = namer.name_assoc_column(ind);
        quote!(let #name = unsafe { self.#table_member_columns.#name.brw(index) } )
    });
    SingleOp {
        op_mod: quote! {
            pub mod #mod_borrow {
                pub struct #mod_borrow_struct_borrow<'brw> {
                    #(#struct_fields),*
                }
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #struct_window<'imm> {
                pub fn #method_borrow<'brw>(&'brw self, key: #type_key) -> Result<#mod_borrow::#mod_borrow_struct_borrow<'brw>, #type_key_error> {
                    let #pulpit_path::column::Entry {index, data: #name_primary_column} = match self.#table_member_columns.#name_primary_column.brw(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(#type_key_error),
                    };
                    #(#assoc_brws;)*

                    Ok(#mod_borrow::#mod_borrow_struct_borrow {
                        #(#borrowed_fields,)*
                    })
                }
            }
        }
        .into(),
    }
}
