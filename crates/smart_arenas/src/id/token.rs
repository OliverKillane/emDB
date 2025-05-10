use std::marker::PhantomInvariantLifetime;

/// A token used to enforce exclusive use of a lifetime (`'id`) by a single
/// instance on an [crate::arena::Arena].
pub struct Token<'id>(PhantomInvariantLifetime<'id>);

pub fn context<R, F: for<'id> FnOnce(Token<'id>) -> R>(cl: F) -> R {
    cl(Token(PhantomInvariantLifetime::new()))
}

/// Used when many [Token]s are required.
/// ```
/// # use smart_arenas::prelude::*;
/// multiple_context!(tk1, tk2, tk3, tk4 => {
///     let arena1 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk1);
///     let arena2 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk2);
///     let arena3 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk3);
///     let arena4 = Own::<Key<u32>, Blocks<32>, u32>::new(0, tk4);
/// });
/// ```
#[macro_export]
macro_rules! multiple_context {
    (=> $code:block ) => {
        {
            $code
        }
    };
    ($token:ident $(,$rest:ident)* => $code:block ) => {
        $crate::id::token::context(|$token| {
            $crate::multiple_context!($($rest),* => $code)
        })
    }
}
