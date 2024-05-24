use super::*;

pub struct PrimaryPull(pub AssocPull);

impl ColKind for PrimaryPull {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        self.0.derives()
    }

    fn generate_base_type(&self, namer: &CodeNamer) -> Tokens<Type> {
        let CodeNamer { pulpit_path, .. } = namer;
        quote!(#pulpit_path::column::PrimaryPull).into()
    }

    fn generate_generics(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> TokenStream {
        let inner_type = self.0.generate_column_type(namer, imm_type, mut_type);
        quote!(<#inner_type>)
    }

    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        self.0.convert_imm(namer, imm_fields)
    }

    fn convert_imm_type(&self, field: &Field, namer: &CodeNamer) -> Tokens<Type> {
        self.0.convert_imm_type(field, namer)
    }

    fn requires_get_lifetime(&self) -> bool {
        self.0.requires_get_lifetime()
    }
}
