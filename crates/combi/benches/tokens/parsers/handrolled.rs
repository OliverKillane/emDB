use proc_macro2::{Delimiter, TokenStream, TokenTree};

use super::super::{LargeGroups, LongSequence, Nothing, Parse, RecursiveIdent};

pub struct HandRolled;

impl Parse<RecursiveIdent> for HandRolled {
    fn parse(input: TokenStream) -> RecursiveIdent {
        let mut tkiter = input.into_iter();
        match tkiter.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == '!' => RecursiveIdent::Final,
            Some(TokenTree::Ident(id)) => match tkiter.next() {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    let recur = HandRolled::parse(g.stream());
                    RecursiveIdent::Next {
                        id: id.to_string(),
                        recur: Box::new(recur),
                    }
                }
                _ => panic!("Expected group {{  }}"),
            },
            _ => panic!("Unexpected token"),
        }
    }
}

impl Parse<LongSequence> for HandRolled {
    fn parse(input: TokenStream) -> LongSequence {
        let mut tkiter = input.into_iter();
        let mut ids = Vec::new();
        for tk in tkiter {
            if let TokenTree::Ident(id) = tk {
                ids.push(id);
            } else {
                panic!("Expected ident")
            }
        }
        LongSequence { ids }
    }
}

impl Parse<Nothing> for HandRolled {
    fn parse(tks: TokenStream) -> Nothing {
        assert!(tks.is_empty());
        Nothing
    }
}

impl Parse<LargeGroups> for HandRolled {
    fn parse(input: TokenStream) -> LargeGroups {
        fn parse_token(tkt: TokenTree) -> LargeGroups {
            match tkt {
                TokenTree::Group(g) => {
                    if g.delimiter() == Delimiter::Parenthesis {
                        let mut puncts = Vec::new();
                        for tk in g.stream() {
                            match tk {
                                TokenTree::Punct(p) => puncts.push(p.as_char()),
                                _ => unreachable!("assumes correct input"),
                            }
                        }
                        LargeGroups::Puncts(puncts)
                    } else if g.delimiter() == Delimiter::Brace {
                        let mut groups = Vec::new();
                        for tk in g.stream() {
                            groups.push(parse_token(tk));
                        }
                        LargeGroups::Groups(groups)
                    } else {
                        unreachable!("assumes correct input")
                    }
                }
                _ => unreachable!("assumes correct input"),
            }
        }

        let mut tkiter = input.into_iter();
        let first = tkiter.next().unwrap();
        let g = parse_token(first);
        assert!(tkiter.next().is_none());
        g
    }
}
