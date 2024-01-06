//! A small text parsing library atop combi that allows for complex recovery, and context sensitivity.
//!
//! <div class="warning">This part of the library is unimplemented, as it is not useful for the core goal of the emdb project</div>
//!
//!
//! Support complex recovery.
//! - Combi already supports recovery that can succeed (e.g. use a different successful parser),
//!   continue or error to fall through to another outer recovery
//! - With text there are more options to backtrack, than with tokenstreams
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
//!             Statement::from
//!         ),
//!         skipto(isletter(','))
//!     ),
//!     letter(';')
//! )
//! ```
//!
//! Support context based parsing fully using select.
//! - Combi's current select is entirely stack allocated, but does use a vtable (we can improve upon this)
//! ```ignore
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
//! Support changes in input stream type.
//! - Combis can have different output that input, so we can map between different representations where convenient.
//! ```ignore
//! diffseq(
//!    tokenize(...),
//!    seq(
//!         word("hello"),
//!         space,
//!     )
//! )
//! ```

unimplemented!();
