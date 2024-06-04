use super::SingleOp;
use crate::{
    groups::{FieldIndex, Groups},
    namer::CodeNamer,
};
use proc_macro2::TokenStream;
use quote::quote;

fn generate_borrow_fields<'a>(
    groups: &'a Groups,
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

pub fn generate(groups: &Groups, namer: &CodeNamer) -> SingleOp {
    let CodeNamer {
        type_key,
        struct_window,
        pulpit_path,
        name_primary_column,
        type_key_error,
        struct_table_member_columns,
        mod_borrow,
        mod_borrow_struct_borrow,
        struct_window_method_borrow,
        name_phantom_member,
        ..
    } = namer;

    let (struct_fields_def, borrowed_fields) = if groups.idents.is_empty() {
        (
            quote!(pub #name_phantom_member: std::marker::PhantomData<&'brw ()>),
            quote!(#name_phantom_member: std::marker::PhantomData),
        )
    } else {
        let borrowed_fields = generate_borrow_fields(groups, namer);
        let struct_fields = groups.idents.iter().map(|(field_name, field_index)| {
            let field_ty = groups.get_type(field_index).unwrap();
            quote!(pub #field_name: &'brw #field_ty)
        });
        (quote! {#(#struct_fields),*}, quote!(#(#borrowed_fields),*))
    };

    let assoc_brws = (0..groups.assoc.len()).map(|ind| {
        let name = namer.name_assoc_column(ind);
        quote!(let #name = unsafe { self.#struct_table_member_columns.#name.assoc_brw(index) } )
    });
    SingleOp {
        op_mod: quote! {
            pub mod #mod_borrow {
                pub struct #mod_borrow_struct_borrow<'brw> {
                    #struct_fields_def
                }
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #struct_window<'imm> {
                pub fn #struct_window_method_borrow<'brw>(&'brw self, key: #type_key) -> Result<#mod_borrow::#mod_borrow_struct_borrow<'brw>, #type_key_error> {
                    let #pulpit_path::column::Entry {index, data: #name_primary_column} = match self.#struct_table_member_columns.#name_primary_column.brw(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(#type_key_error),
                    };
                    #(#assoc_brws;)*

                    Ok(#mod_borrow::#mod_borrow_struct_borrow {
                        #borrowed_fields
                    })
                }
            }
        }
        .into(),
    }
}
