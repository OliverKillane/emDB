# Quote Debug
A crate containing a simple type for checking the syntax of produced tokenstreams when built with [`debug_assertions`](https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions).

Incurs no overhead when built optimised ([`debug_assertions`](https://doc.rust-lang.org/reference/conditional-compilation.html#debug_assertions) are disabled).

To get access to the all the `syn` types, `syn` needs the `"full"` feature enabled.

For example the following pass
```rust
use quote_debug::Tokens;
use quote::quote;
use syn::{ExprBlock, ItemEnum, TraitItemFn};

Tokens::<ExprBlock>::from(quote! {
    {
        block_of_code();
        let y = 3;
    }
});
Tokens::<TraitItemFn>::from(quote! {
    /// stuff
    fn method(&self) -> i32 {
        
    }
});
Tokens::<ItemEnum>::from(quote! {
    enum Cool {
        A, B, C
    }
});
```

While generating invalid syntax fails.
```rust,should_panic
use quote_debug::Tokens;
use quote::quote;
use syn::ExprBlock;

Tokens::<ExprBlock>::from(quote! {
    not_in_block; {
        block_of_code();
        let y = 3;
    }
}); // Panic! 
```
