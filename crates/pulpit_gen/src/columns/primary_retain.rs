use super::*;

pub struct PrimaryRetain {
    pub block_size: usize,
}

impl ColKind for PrimaryRetain {
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
        let Self { block_size } = self;
        let pulpit_path = namer.pulpit_path();
        quote! { #pulpit_path::column::PrimaryRetain<#imm_type, #mut_type, #block_size> }.into()
    }

    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        let field_defs = imm_fields.iter().map(|Field { name, ty }| {
            quote! {
                pub #name : &'imm #ty
            }
        });

        let unpacked_name = namer.mod_columns_struct_imm_unpacked();
        let unpacking_fn = namer.mod_columns_fn_imm_unpack();
        let imm_name = namer.mod_columns_struct_imm();

        let fields = imm_fields.iter().map(|Field { name, ty }| name);
        let unpack_fields = fields.clone();

        ImmConversion {
            imm_unpacked: quote!{
                pub struct #unpacked_name<'imm> {
                    #(#field_defs),*
                }
            }.into(),
            unpacker:  quote!{
                pub fn #unpacking_fn<'imm>(#imm_name { #(#fields),* }: &'imm #imm_name) -> #unpacked_name<'imm> {
                    #unpacked_name { #(#unpack_fields),* }
                }
            }.into()
        }
    }

    fn generate_column_type_no_generics(&self, namer: &CodeNamer) -> Tokens<Type> {
        let pulpit_path = namer.pulpit_path();
        quote! { #pulpit_path::column::PrimaryRetain }.into()
    }

    fn requires_get_lifetime(&self) -> bool {
        true
    }

    fn convert_imm_type(&self, field: &Field, namer: &CodeNamer) -> Tokens<Type> {
        let ty = &field.ty;
        let lifetime = namer.lifetime_imm();
        quote!(&#lifetime #ty).into()
    }
}
