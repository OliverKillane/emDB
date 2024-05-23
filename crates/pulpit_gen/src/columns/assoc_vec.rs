use super::*;

pub struct AssocVec;

impl ColKind for AssocVec {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn generate_column_type(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> Tokens<Type> {
        let pulpit_path = &namer.pulpit_path;
        quote! { #pulpit_path::column::AssocVec<#imm_type, #mut_type> }.into()
    }

    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        let field_defs = imm_fields.iter().map(|Field { name, ty }| {
            quote! {
                pub #name : #ty
            }
        });
        let fields = imm_fields.iter().map(|Field { name, ty: _ }| name);
        let unpack_fields = fields.clone();

        let CodeNamer {
            mod_columns_struct_imm_unpacked,
            mod_columns_fn_imm_unpack,
            mod_columns_struct_imm,
            ..
        } = namer;

        ImmConversion {
            imm_unpacked: quote! {
                pub struct #mod_columns_struct_imm_unpacked {
                    #(#field_defs),*
                }
            }
            .into(),
            unpacker: quote! {
                pub fn #mod_columns_fn_imm_unpack(#mod_columns_struct_imm { #(#fields),* }: #mod_columns_struct_imm) -> #mod_columns_struct_imm_unpacked {
                    #mod_columns_struct_imm_unpacked { #(#unpack_fields),* }
                }
            }
            .into(),
        }
    }

    fn generate_column_type_no_generics(&self, namer: &CodeNamer) -> Tokens<Type> {
        let pulpit_path = &namer.pulpit_path;
        quote! { #pulpit_path::column::AssocVec }.into()
    }

    fn requires_get_lifetime(&self) -> bool {
        false
    }

    fn convert_imm_type(&self, field: &Field, _: &CodeNamer) -> Tokens<Type> {
        field.ty.clone().into()
    }
}
