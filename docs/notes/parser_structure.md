combinators required:



parsers:
```rust
// Parser<TokenIter>
getident()
getpunct()
matchident("something")
matchpunct(';')
terminal()
ingroup()
ahead(Parser<&TokenTree>)

skip_until(Parser<&TokenTree>)
skip_past(Parser<&TokenTree>)

syn_until<T: Parser>(Parser<&TokenTree>)

// Parser<&TokenTree>
isident(tt)
isgroup()
ispunct(tt)

// derived
punctlist(',', parse)
```

```rust
let emdb_parser =
    seq(
        name_parser,
        many0(
            nonempty(),
            ident_peek_choose( |t|
                match t {
                    "table" => table_parser,
                    "query" => query_parser,
                    _ => error("Expected table or query")  
                }
            )
        )
    );

let name_parser = 
    recover(
        map_suc(
            seq(
                ident("name"), 
                seq(
                    getident(), 
                    punct(';')
                )
            ), |(_, (name, _))| name
        ), 
        upto_including_punct(';')
    );

// table { members } @ constraints
let table_parser = 
    map_suc(
        seq(
            ident("table"),
            seq(
                getident(),
                seq(
                    table_members,
                    choice(
                        peek_punct('@'),
                        table_constraints,
                        map_suc(nothing(), |_| todo!()) 
                    ),
                )
            )
        ),
        | (_, (name, (fields, constraints))) | Table{name, fields, constraints}
    );

// name: type [, name: type ]*
fn param_list() -> impl Parser<...> { 
    choice(
        isempty(),
        map_suc(nothing(), |()| vec![]),
        many1(
            choice(
                isempty(),
                map_suc(nothing(), false)
                map_suc(punct(','), |()| true),
            ),
            map_suc(
                recover(
                    seq(
                        getident(),
                        seq(
                            punct(':'),
                            syn::<Type>(peek_punct(','))
                        )
                    ),
                    upto_not_including_punct(',')
                )
                |(name, (_, data_type))| Field{name, data_type} 
            )
        ),
    )
}

let table_members =
    recover(
        group(
            delim::Bracket,
            param_list()
        ),
        immediate()
    );

let table_constraints =
    map_suc(
        seq(
            punct('@'),
            many1(
                choice(
                    peek_punct(';'),
                    map_suc(punct(';'), |()| false),
                    map_suc(punct(','), |()| true)
                ),
                fn_brack
            ),
        ),
        |(_, cons)| cons
    );

let fn_brack = todo!();

let query_parser = 
    seq(
        ident("query"),
        seq(
            query_params, 
            query_body,
        )
    );

let query_params = 
    recover(
        group(
            Delim::Paren,
            param_list()
        ),
        immediate()
    );

// ret
// ref table
// let var
        // ident_peek_choose(|i| 
        //     match i {
        //         "ret" => map_suc(seq(ident("ret"), punct(';')), |(i, )| Ops::Return(i.span())),
        //         "let" => map_suc(seq(ident("let"), seq(get_ident(), punct(';'))), |(_, (i, _))| Ops::Let(i)),
        //         op if ops.contains(op) => parse_op,
        //         name => map_suc(get_ident(), )
        //     }
        // ),
// operator(..insides..)
// table
let stream = 
    recover(
        recursive(|r|
            conditions!(
                peek_ident("ret"),
                map_suc(seq(ident("ret"), punct(';')), |(i, _)| Ops::Return(i.span())),
                choice(
                    peek_ident()
                    map_suc(seq(ident("let"), seq(get_ident(), punct(';'))), |()|)
                )
            )
        )
        upto_including_punct(';')
    )

let query_body = 
    recover(
        group(
            Delim::Bracket,
            choice(
                isempty(),

            )
        ),
        immediate()
    )