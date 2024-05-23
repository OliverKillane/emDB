use super::SingleOp;
use crate::{
    columns::{ColKind, PrimaryKind},
    groups::{Field, FieldIndex, Group, Groups, MutImmut},
    namer::CodeNamer,
};
use proc_macro2::TokenStream;
use quote::quote;

fn generate_get_fields<'a, Primary: PrimaryKind>(
    groups: &'a Groups<Primary>,
    namer: &'a CodeNamer,
) -> impl Iterator<Item = TokenStream> + 'a {
    groups.idents.iter().map(|(field_name, field_index)| {
        let data = match field_index {
            FieldIndex::Primary(_) => namer.name_primary_column(),
            FieldIndex::Assoc { assoc_ind, inner } => namer.name_assoc_column(*assoc_ind),
        };

        let imm_access = if field_index.is_imm() {
            quote!(imm_data)
        } else {
            quote!(mut_data)
        };

        quote!(#field_name: #data.#imm_access.#field_name)
    })
}

pub fn generate_get_struct_fields<'a, Primary: PrimaryKind>(
    groups: &'a Groups<Primary>,
    namer: &'a CodeNamer,
) -> Vec<TokenStream> {
    fn append<Col: ColKind>(
        fs: &mut Vec<TokenStream>,
        col: &Col,
        fields: &MutImmut<Vec<Field>>,
        namer: &CodeNamer,
    ) {
        for Field { name, ty } in &fields.mut_fields {
            fs.push(quote!(pub #name: #ty));
        }
        for field @ Field { name, .. } in &fields.imm_fields {
            let ty_trans = col.convert_imm_type(field, namer);
            fs.push(quote!(pub #name: #ty_trans));
        }
    }
    let mut def_fields = Vec::with_capacity(groups.idents.len());
    append(
        &mut def_fields,
        &groups.primary.col,
        &groups.primary.fields,
        namer,
    );
    for Group { col, fields } in &groups.assoc {
        append(&mut def_fields, col, fields, namer);
    }
    def_fields
}

pub fn generate<Primary: PrimaryKind>(groups: &Groups<Primary>, namer: &CodeNamer) -> SingleOp {
    let type_key_error = namer.type_key_error();
    let type_key = namer.type_key();
    let struct_window = namer.struct_window();
    let pulpit_path = namer.pulpit_path();
    let name_primary_column = namer.name_primary_column();
    let table_member_columns = namer.table_member_columns();
    let mod_columns = namer.mod_columns();
    let mod_columns_fn_imm_unpack = namer.mod_columns_fn_imm_unpack();
    let mod_get = namer.mod_get();
    let mod_get_struct_get = namer.mod_get_struct_get();
    let trait_get = namer.trait_get();

    let include_lifetime = groups.primary.col.requires_get_lifetime()
        || groups
            .assoc
            .iter()
            .any(|Group { col, fields: _ }| col.requires_get_lifetime()); // TODO: implement
    let lifetime = if include_lifetime {
        let lf = namer.lifetime_imm();
        quote!(<#lf>)
    } else {
        quote!()
    };

    let get_struct_fields = generate_get_struct_fields(groups, namer);

    let assoc_cols = (0..groups.assoc.len()).map(|ind| {
        let name = namer.name_assoc_column(ind);
        quote!(let #name = unsafe { self.#table_member_columns.#name.get(index) }.convert_imm(#mod_columns::#name::#mod_columns_fn_imm_unpack))
    });
    let get_fields = generate_get_fields(groups, namer);

    SingleOp {
        op_mod: quote! {
            pub mod #mod_get {
                pub struct #mod_get_struct_get #lifetime {
                    #(#get_struct_fields,)*
                }
            }
        }
        .into(),
        op_trait: quote! {
            pub trait #trait_get #lifetime {
                fn get(&self, key: #type_key) -> Result<#mod_get::#mod_get_struct_get #lifetime, #type_key_error>;
            }
        }
        .into(),
        op_impl: quote! {
            impl <'imm> #trait_get #lifetime for #struct_window<'imm> {
                fn get(&self, key: #type_key) -> Result<#mod_get::#mod_get_struct_get #lifetime, #type_key_error> {
                    let #pulpit_path::column::Entry {index, data: #name_primary_column} = match self.#table_member_columns.#name_primary_column.get(key) {
                        Ok(entry) => entry,
                        Err(e) => return Err(#type_key_error),
                    };
                    let #name_primary_column = #name_primary_column.convert_imm(#mod_columns::#name_primary_column::#mod_columns_fn_imm_unpack);
                    #(#assoc_cols;)*

                    Ok(#mod_get::#mod_get_struct_get {
                        #(#get_fields,)*
                    })
                }
            }
        }
        .into(),
    }
}
