//! A small text parsing library that allows for complex recovery, and context sensitivity.
//!
//! <div class="warning">This part of the library is unimplemented, as it is not useful for the core goal of the emdb project</div>
//!
//! ```ignore
//! // try to parse "hello",
//! // if unable re-try with "hullo",
//! // if that fails skip to the space and continue,
//! // if that fails then fail parsing
//! // then parse a space and "world"
//! seqs!(
//!     recover(
//!         word("hello"),
//!         backtrack(
//!             recover(
//!                 word("hullo"),
//!                 skipto(isspace),
//!             )
//!         )
//!     ),
//!     space,
//!     word("world"),
//! )
//! ```
//!
//! ```ignore
//! // Support context based parsing (e.g. C++)
//! // for example
//! select(
//!     getword(),
//!     (english, french, german, dutch),
//!     |txt, (english, french, german, dutch)| match &txt {
//!         "hello" => english,
//!         "bonjour" => french,
//!         "hallo" => recover(
//!             german,
//!             backtrack(dutch)
//!         )
//!     },
//!     |_| "english, french, dutch or german",
//! )
//! ```
//!
//! // fallthrough recovery
//! ```ignore
//! // can recover inside , lists, or outer list of statements ,,,;,,,;
//! seplist(
//!     recover(
//!         mapsuc(
//!             seplist(
//!                 recover(
//!                     item_parse(),
//!                     skipto(
//!                         or(
//!                             // if at ;, fail this recovery and fall through to the next
//!                             errortrue(isletter(';')),
//!                             isletter(',')
//!                         )
//!                     )
//!                 ),
//!                 letter(','),
//!             ),
//!             |s| Statement::from(s)
//!         ),
//!         skipto(isletter(','))
//!     ),
//!     letter(';')
//! )
//! ```

unimplemented!();
