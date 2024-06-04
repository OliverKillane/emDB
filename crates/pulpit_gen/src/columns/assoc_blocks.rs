use super::*;

/// An append only column of blocks, with pointer stability.
/// No restrictions on the mutable and immutable
pub struct AssocBlocks {
    pub block_size: usize,
}

impl ColKind for AssocBlocks {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        let CodeNamer {
            mod_columns_struct_imm_unpacked,
            mod_columns_fn_imm_unpack,
            mod_columns_struct_imm,
            name_phantom_member,
            ..
        } = namer;

        if imm_fields.is_empty() {
            ImmConversion {
                imm_unpacked: quote!{
                    pub struct #mod_columns_struct_imm_unpacked<'imm> {
                        pub #name_phantom_member: std::marker::PhantomData<&'imm ()>
                    }
                }.into(),
                unpacker:  quote!{
                    pub fn #mod_columns_fn_imm_unpack<'imm>(_: &'imm #mod_columns_struct_imm) -> #mod_columns_struct_imm_unpacked<'imm> {
                        #mod_columns_struct_imm_unpacked { #name_phantom_member: std::marker::PhantomData }
                    }
                }.into()
            }
        } else {
            let field_defs = imm_fields.iter().map(|Field { name, ty }| {
                quote! {
                    pub #name : &'imm #ty
                }
            });
            let fields = imm_fields.iter().map(|Field { name, ty: _ }| name);
            let unpack_fields = fields.clone();

            ImmConversion {
                imm_unpacked: quote!{
                    pub struct #mod_columns_struct_imm_unpacked<'imm> {
                        #(#field_defs),*
                    }
                }.into(),
                unpacker:  quote!{
                    pub fn #mod_columns_fn_imm_unpack<'imm>(#mod_columns_struct_imm { #(#fields),* }: &'imm #mod_columns_struct_imm) -> #mod_columns_struct_imm_unpacked<'imm> {
                        #mod_columns_struct_imm_unpacked { #(#unpack_fields),* }
                    }
                }.into()
            }
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let pulpit_path = &namer.pulpit_path;
        quote! { #pulpit_path::column::AssocBlocks }.into()
    }

    fn generate_generics(
        &self,
        _namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> TokenStream {
        let Self { block_size } = self;
        quote! { <#imm_type, #mut_type, #block_size> }
    }

    fn requires_get_lifetime(&self) -> bool {
        true
    }

    fn convert_imm_type(&self, field: &Field, namer: &CodeNamer) -> Tokens<Type> {
        let ty = &field.ty;
        let lifetime = &namer.lifetime_imm;
        quote!(&#lifetime #ty).into()
    }

    fn check_column_application(
        &self,
        error_span: Span,
        imm_fields: &[Field],
        mut_fields: &[Field],
        transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic> {
        if deletions {
            LinkedList::from([Diagnostic::spanned(
                error_span,
                Level::Error,
                String::from("AssocBlocks does not support deletions"),
            )])
        } else {
            LinkedList::new()
        }
    }
}
