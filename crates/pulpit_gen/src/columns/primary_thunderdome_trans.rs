use super::*;

/// A column that uses `thunderdome`
pub struct PrimaryThunderDomeTrans;

impl ColKind for PrimaryThunderDomeTrans {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote! { #pulpit_path::column::PrimaryThunderDome }.into()
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
