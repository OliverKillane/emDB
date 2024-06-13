use super::*;

/// an arena that supports deletions/pulls, but stores immutable data in a separate (pointer stable) arena.
/// - Can use pointer to the immutable data to get the mutable data.
pub struct PrimaryRetainCopy {
    pub block_size: usize,
}

impl ColKind for PrimaryRetainCopy {
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
            ..
        } = namer;

        if imm_fields.is_empty() {
            unreachable!("Cannot run on empty fields")
        } else {
            let field_defs = imm_fields.iter().map(|Field { name, ty }| {
                quote! {
                    pub #name : #ty
                }
            });

            let fields = imm_fields.iter().map(|Field { name, ty: _ }| name);
            let unpack_fields = fields.clone();

            ImmConversion {
                imm_unpacked: quote!{
                    pub struct #mod_columns_struct_imm_unpacked {
                        #(#field_defs),*
                    }
                }.into(),
                unpacker:  quote!{
                    pub fn #mod_columns_fn_imm_unpack<'imm>(#mod_columns_struct_imm { #(#fields),* }: &'imm #mod_columns_struct_imm) -> #mod_columns_struct_imm_unpacked {
                        #mod_columns_struct_imm_unpacked { #(#unpack_fields: #unpack_fields.clone()),* }
                    }
                }.into()
            }
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let pulpit_path = &namer.pulpit_path;
        quote! { #pulpit_path::column::PrimaryRetain }.into()
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
        false
    }

    fn check_column_application(
        &self,
        _error_span: Span,
        imm_fields: &[Field],
        _mut_fields: &[Field],
        _transactions: bool,
        _deletions: bool,
    ) -> LinkedList<Diagnostic> {
        if imm_fields.is_empty() {
            LinkedList::from([Diagnostic::new(
                Level::Error,
                String::from("PrimaryRetain requires at least one immutable field"),
            )])
        } else {
            LinkedList::new()
        }
    }
}
