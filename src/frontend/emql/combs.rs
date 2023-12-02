// //! Combinators for building TokenTree parsers.
// //! - Can only advance the iterator
// //! - Can reach the end of the tokenstream
// //! - Need error recovery
// //! - Must produce an AST with errors

// /// Basic parser trait
// /// - always produce a value
// /// - always recover
// /// - always advance the iterator
// trait Parser<const BACKTRACK: usize, I: Iterator, O> {
//     fn parse(self, iter: IteratorState<BACKTRACK, I>) -> Result<O, impl ParseResolve>;
//     fn backtrack();
//     fn recover();
// }

// trait ParseResolve {
//     fn backtrack();
//     fn recover();
// }

// enum ParseResult<O> {
//     Ok(O),
//     Err(ParseResolve),
// }

// struct IteratorState<const BACK: usize, I: Iterator> {
//     back: [I::Item; BACK],
//     iter: I,
// }

// fn then(parser1, parser2) -> impl Parser {
//     ||
// }

// fn test() {
//     let parser = then(
//         text("name"),
//         then(
//             string(),
//             then(
//                 group(
//                     ...,
//                     terminal()
//                 ),
//                 terminal(),
//             ),
//         )
//     );

//     parser.parse(iter)
// }
