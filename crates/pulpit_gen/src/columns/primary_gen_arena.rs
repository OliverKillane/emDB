use super::*;

pub struct PrimaryGenArena;

impl ColKind for PrimaryGenArena {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields: vec![Ident::new("Clone", Span::call_site())],
        }
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote! { #pulpit_path::column::PrimaryGenerationalArena }.into()
    }

    fn check_column_application(
        &self,
        error_span: Span,
        _imm_fields: &[Field],
        _mut_fields: &[Field],
        transactions: bool,
        _deletions: bool,
    ) -> LinkedList<Diagnostic> {
        if transactions {
            LinkedList::from([Diagnostic::spanned(
                error_span,
                Level::Error,
                String::from("primaryGenArena does not support transactions"),
            )])
        } else {
            LinkedList::new()
        }
    }
}
