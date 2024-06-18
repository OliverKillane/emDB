use super::*;

/// A column that uses `thunderdome`
pub struct AssocAppVec;

impl ColKind for AssocAppVec {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote! { #pulpit_path::column::AssocAppVec }.into()
    }

    fn check_column_application(
        &self,
        error_span: Span,
        _imm_fields: &[Field],
        _mut_fields: &[Field],
        _transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic> {
        let mut errors = LinkedList::new();
        if deletions {
            errors.push_back(Diagnostic::spanned(
                error_span,
                Level::Error,
                String::from("AssocAppVec does not support deletions"),
            ))
        }
        errors
    }
}
