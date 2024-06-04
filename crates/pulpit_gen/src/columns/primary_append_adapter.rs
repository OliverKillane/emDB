use super::*;

pub struct PrimaryAppendAdapter;

impl ColKind for PrimaryAppendAdapter {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![],
            mut_fields: vec![],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote!(#pulpit_path::column::PrimaryAppendAdapter).into()
    }

    fn check_column_application(
        &self,
        error_span: Span,
        imm_fields: &[Field],
        mut_fields: &[Field],
        transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic> {
        imm_fields
            .iter()
            .chain(mut_fields.iter())
            .map(|Field { name, ty: _ }| {
                Diagnostic::spanned(
                    name.span(),
                    Level::Error,
                    String::from("PrimaryAppendAdapter takes no fields"),
                )
            })
            .collect()
    }
}
