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
        error_span: Span,
        imm_fields: &[Field],
        mut_fields: &[Field],
        transactions: bool,
        deletions: bool,
    ) -> LinkedList<Diagnostic> {
        LinkedList::new()
    }
}
