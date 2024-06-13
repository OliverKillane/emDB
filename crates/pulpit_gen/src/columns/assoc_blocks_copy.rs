use super::*;

/// A variant of [super::AssocBlocks] that does not allow for the immutability optimisation.
/// - This table is for use in supporting benchmarks only.
pub struct AssocBlocksCopy {
    pub block_size: usize,
}

impl ColKind for AssocBlocksCopy {
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

        let field_defs = imm_fields.iter().map(|Field { name, ty }| {
            quote! {
                pub #name: #ty
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
        false
    }

    fn check_column_application(
        &self,
        error_span: Span,
        _imm_fields: &[Field],
        _mut_fields: &[Field],
        _transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic> {
        if deletions {
            LinkedList::from([Diagnostic::spanned(
                error_span,
                Level::Error,
                String::from("AssocBlocksNoCopy does not support deletions"),
            )])
        } else {
            LinkedList::new()
        }
    }
}
