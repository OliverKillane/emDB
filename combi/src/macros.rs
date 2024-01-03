//! Helper macros for parst combinators

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

#[macro_export]
macro_rules! choices {
    (otherwise => $q:expr) => {$q};
    ($p:expr => $q:expr , $($ts:tt)+) => {
        choice($p, $q, choices!($($ts)+))
    };
}

pub use choices;
