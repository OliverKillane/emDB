//! Helper macros for parst combinators

/// ```ignore
/// seqs!(A, B, C, D) = seq(A, seq(B, seq(C, D)));
/// ```
macro_rules! seqs {
    ($p:expr) => {
        $p
    };
    ($p:expr , $($ts:tt)+) => {
        seq($p, seqs!())
    };
}

pub(crate) use seqs;

/// ```ignore
/// choice! {
///     peek("a") => parse_a,
///     peek("b") => parse_b,
///     peek("c") => parse_c,
///     otherwise => parse_otherwise,
/// } = either(
///     peek("a"),
///     parse_a,
///     either(
///         peek("b"),
///         parse_b,
///         either(
///             peek("c"),
///             parse_c,
///             parse_otherwise,
///        ),
///     ),
/// )
/// ```
macro_rules! choice {
    (otherwise => $q:expr) => {$q};
    ($p:expr => $q:expr $(, $ts:tt)+) => {
        either($p, $q, choice!($($ts)+))
    };
}

pub(crate) use choice;
