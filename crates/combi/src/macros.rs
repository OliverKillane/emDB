//! Helper macros

/// Helper to combine deeply nested sequences.
/// ```ignore
/// seq(P1, seq(P2, seq(P3, P4)))
/// // is equivalent to
/// seqs!(P1, P2, P3, P4)
/// ```
#[macro_export]
macro_rules! seqs {
    ($p:expr) => {
        $p
    };
    ($p:expr , $($ts:tt)+) => {
        seq($p, seqs!($($ts)+))
    };
}

pub use seqs;

/// Helper to combine deeply nested choices.
/// ```ignore
///
/// choice(P1, Q1, choice(P2, Q2, choice(P3, Q3, QO)))
/// // is equivalent to
/// choices!{
///     P1        => Q1,
///     P2        => Q2,
///     P3        => Q3,
///     otherwise => QO,
/// }
/// ```
#[macro_export]
macro_rules! choices {
    (otherwise => $q:expr) => {$q};
    ($p:expr => $q:expr , $($ts:tt)+) => {
        choice($p, $q, choices!($($ts)+))
    };
}

pub use choices;
