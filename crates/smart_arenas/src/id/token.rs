use std::marker::PhantomInvariantLifetime;

/// A token used to enforce exclusive use of a lifetime (`'id`) by a single
/// instance on an [crate::arena::Arena].
pub struct Token<'id>(PhantomInvariantLifetime<'id>);

pub fn context<R, F: for<'id> FnOnce(Token<'id>) -> R>(cl: F) -> R {
    cl(Token(PhantomInvariantLifetime::new()))
}
