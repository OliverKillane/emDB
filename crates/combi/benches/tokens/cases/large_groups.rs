use super::super::Parseable;
use proc_macro2::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LargeGroups {
    Groups(Vec<LargeGroups>),
    Puncts(Vec<char>),
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub depth: usize,
    pub branch_width: usize,
    pub leaf_width: usize,
}

impl Parseable for LargeGroups {
    type Param = Size;

    fn generate_case(
        Size {
            depth,
            branch_width,
            leaf_width,
        }: Self::Param,
    ) -> Self {
        if depth == 0 {
            LargeGroups::Puncts(vec!['!'; leaf_width])
        } else {
            let mut groups = Vec::new();
            for _ in 0..branch_width {
                groups.push(LargeGroups::generate_case(Size {
                    depth: depth - 1,
                    branch_width,
                    leaf_width,
                }));
            }
            LargeGroups::Groups(groups)
        }
    }

    fn generate_tokens(&self) -> TokenStream {
        match self {
            LargeGroups::Groups(groups) => {
                let groups = groups.iter().map(Parseable::generate_tokens);
                TokenTree::Group(Group::new(Delimiter::Brace, quote! { #(#groups)* })).into()
            }
            LargeGroups::Puncts(puncts) => {
                let puncts = puncts.iter().map(|c| Punct::new(*c, Spacing::Alone));
                TokenTree::Group(Group::new(Delimiter::Parenthesis, quote! { #(#puncts)* })).into()
            }
        }
    }
}
