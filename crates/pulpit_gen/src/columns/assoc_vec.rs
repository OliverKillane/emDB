use super::*;


pub struct AssocVec;

impl ColKind for AssocVec {
    fn derives(&self) -> MutImmut<Vec<Ident>> {
        MutImmut {
            imm_fields: vec![Ident::new("Clone", Span::call_site())],
            mut_fields:  vec![Ident::new("Clone", Span::call_site())],
        }
    }


    fn generate_column_type(
        &self,
        namer: &CodeNamer,
        imm_type: Tokens<Type>,
        mut_type: Tokens<Type>,
    ) -> Tokens<Type> {
        let pulpit_path = namer.pulpit_path();
        quote!{ #pulpit_path::column::AssocVec<#imm_type, #mut_type> }.into()
    }

    fn convert_imm(&self, namer: &CodeNamer, imm_fields: &[Field]) -> ImmConversion {
        let field_defs = imm_fields.iter().map(|Field { name, ty }| {
            quote!{
                #name : #ty
            }
        });
        let fields = imm_fields.iter().map(|Field { name, ty }| {
            name
        });
        let unpack_fields = fields.clone();  

        let unpacked_name = namer.mod_columns_struct_imm_unpacked();
        let unpacking_fn = namer.mod_columns_fn_imm_unpack();
        let imm_name = namer.mod_columns_struct_imm();

        ImmConversion {
            imm_unpacked: quote!{
                pub struct #unpacked_name {
                    #(#field_defs),*
                }
            }.into(), 
            unpacker:  quote!{
                fn #unpacking_fn(#imm_name { #(#fields),* }: #imm_name) -> #unpacked_name {
                    #unpacked_name { #(#unpack_fields),* }
                }
            }.into()
        }
    }
    
    fn generate_column_type_no_generics(
        &self,
        namer: &CodeNamer,
    ) -> Tokens<Type> {
        let pulpit_path = namer.pulpit_path();
        quote!{ #pulpit_path::column::AssocVec }.into()
    }
}