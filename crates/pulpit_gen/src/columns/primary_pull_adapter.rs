use super::*;

pub struct PrimaryPullAdapter;

impl ColKind for PrimaryPullAdapter {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![],
            mut_fields: vec![],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote!(#pulpit_path::column::PrimaryPullAdapter).into()
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
            .map(|Field { name, ty:_ }| {
                Diagnostic::spanned(
                    name.span(),
                    Level::Error,
                    String::from("PrimaryPullAdapter takes no fields"),
                )
            })
            .collect()
    }
}
