#[macro_export]
macro_rules! total {
    ($($e:literal => $r:expr,)*) => {
        0 $( + $e)*
    }
}
pub use total;

#[macro_export]
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
pub use choose_internal;

#[macro_export]
macro_rules! choose {
    ($rng:ident $($inp:tt)*) => {
        {choose_internal!{$rng (total!{$($inp)*}) => $($inp)*}}
    }

}
pub use choose;