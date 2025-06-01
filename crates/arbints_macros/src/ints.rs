use std::iter::once;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote_debug::Tokens;
use syn::{Ident, ItemImpl, ItemType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntPrototype {
    pub signed: bool,
    pub size: usize,
}

struct Bounds {
    min: isize,
    max: isize,
}

impl IntPrototype {
    pub fn all() -> impl Iterator<Item = Self> {
        (15..17).flat_map(|size| {
            once(IntPrototype { signed: true, size }).chain(once(IntPrototype {
                signed: false,
                size,
            }))
        })
    }

    pub fn alias(&self) -> Ident {
        let sign = if self.signed { "i" } else { "u" };
        Ident::new(&format!("{sign}{}", self.size), Span::call_site()).into()
    }

    fn alias_def(&self) -> Option<Tokens<ItemType>> {
        let alias = self.alias();
        let Self { signed, size } = *self;

        if !self.is_std() {
            Some(
                quote! {
                    pub type #alias = Int<#size, #signed>;
                }
                .into(),
            )
        } else {
            None
        }
    }

    fn underlying(&self) -> Ident {
        let size = match self.size {
            1..=8 => 8,
            9..=16 => 16,
            17..=32 => 32,
            33..=64 => 64,
            65..=128 => 128,
            _ => panic!("Unsupported bit size: {}", self.size),
        };
        let sign = if self.signed { "i" } else { "u" };
        Ident::new(&format!("{sign}{size}"), Span::call_site())
    }

    fn bounds(&self) -> Bounds {
        if self.signed {
            let min_neg = 2isize.pow((self.size as u32) - 1) as isize;
            let min = -min_neg;
            let max = min_neg - 1;
            Bounds { min, max }
        } else {
            Bounds {
                min: 0,
                max: (2isize.pow(self.size as u32) as isize) - 1,
            }
        }
    }

    fn is_std(&self) -> bool {
        match self.size {
            8 | 16 | 32 | 64 | 128 => true,
            _ => false,
        }
    }

    fn integer_impl(&self) -> Tokens<ItemImpl> {
        let underlying = self.underlying();
        let Self { signed, size } = *self;
        let Bounds { min, max } = self.bounds();
        let alias = self.alias();

        let (min_val, max_val) = if self.is_std() {
            (quote!(#alias::MIN), quote!(#alias::MAX))
        } else {
            (quote!(Self(#min as #underlying)), quote!(Self(#max as #underlying)))
        };

        quote! {
            impl Integer for #alias {
                const SIZE: usize = #size;
                const SIGNED: bool = #signed;
                const MAX: Self = #max_val;
                const MIN: Self = #min_val;
                type Inner = #underlying;
            }
        }
        .into()
    }

    fn conversion(&self, other: &Self) -> Option<Tokens<ItemImpl>> {
        let Bounds { min, max } = self.bounds();
        let other_underlying = other.underlying();
        let other_alias = other.alias();
        let self_alias = self.alias();

        let check_min = min > other.bounds().min;
        let check_max = max < other.bounds().max;

        if (self.is_std() && other.is_std()) || self == other {
            None
        } else {
            let ret_val = if self.is_std() {
                quote!(res)
            } else {
                quote!(Self(res))
            };

            Some(
                if !(check_max || check_min) {
                    quote! {
                        impl From<#other_alias> for #self_alias {
                            fn from(value: #other_alias) -> Self {
                                let res = value.extract() as <Self as Integer>::Inner;
                                #ret_val
                            }
                        }
                    }
                } else {
                    quote! {
                        impl TryFrom<#other_alias> for #self_alias {
                            type Error = TryFromError;

                            fn try_from(value: #other_alias) -> Result<Self, Self::Error> {
                                let value_extracted = value.extract();
                                if value_extracted > (#max as #other_underlying) {
                                    Err(TryFromError)
                                } else if value_extracted < (#min as #other_underlying) {
                                    Err(TryFromError)
                                } else {
                                    let res = value_extracted as <Self as Integer>::Inner;
                                    Ok(#ret_val)
                                }
                            }
                        }
                    }
                }
                .into(),
            )
        }
    }

    pub fn generate(&self) -> TokenStream {
        let alias_def = self.alias_def();
        let int_impl = self.integer_impl();
        let conversion = Self::all().filter_map(|other| self.conversion(&other));
        quote! {
            #alias_def
            #int_impl
            #(#conversion)*
        }
    }
}
