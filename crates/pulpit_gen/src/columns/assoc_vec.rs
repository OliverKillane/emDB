use super::*;

pub struct AssocVec;

impl ColKind for AssocVec {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let pulpit_path = &namer.pulpit_path;
        quote! { #pulpit_path::column::AssocVec }.into()
    }

    fn check_column_application(
        &self,
        _error_span: Span,
        _imm_fields: &[Field],
        _mut_fields: &[Field],
        _transactions: bool,
        _deletions: bool,
    ) -> LinkedList<Diagnostic> {
        LinkedList::new()
    }
}
