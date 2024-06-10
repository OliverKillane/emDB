//! # A Basic Statement for Choice
//!
//! ```ignore
//! use rand;
//! fn foo() {
//!     let mut rng = rand::thread_rng();
//!
//!     // 50% for true, 50% for false
//!     let x = choose! {
//!         rng;
//!         1 => true,
//!         1 => false,
//!     };
//! }
//! ```
#![allow(unused_macros, unused_imports)]
macro_rules! total {
    ($($e:literal => $r:expr,)*) => {
        0 $( + $e)*
    }
}

macro_rules! choose_internal {
    ($rng:ident $total:expr => $e:literal => $r:expr,) => {
        $r
    };
    ($rng:ident $total:expr => $e:literal => $r:expr, $($rest:tt)+) => {
        if $rng.gen_ratio($e, $total) {
            $r
        } else {
            choose_internal!($rng ($total - $e) => $($rest)+ )
        }
    };
}

macro_rules! choose {
    ($rng:ident ; $($inp:tt)*) => {
        choose_internal!{$rng (total!{$($inp)*}) => $($inp)*}
    }
}

pub(crate) use choose;

// all need to be public for macro expansion
pub(crate) use choose_internal;
pub(crate) use total;
