# Enumtrait
A crate for deriving enum based polymorphism.

## Pattern
Often we want to implement traits separately for each variant of a enum.
This is traditionally implemented in two ways:
1. By manually writing the enum of `Typename(TypeName)` variants, and implementing the trait on that enum with match statements.
2. Using `dyn` with either references, or heap allocating (classic `Box<dyn trait>`)

The former is tedious. The later can require heap allocation, and the cost of a virtual call.

This crate removes the tediousness of the former.
```rust
use enumtrait;

struct Bar { bar_field: usize }
struct Bing { bing_field: usize, other_field: String }

#[enumtrait::quick_enum]
#[enumtrait::store(foo_macro_store)]
enum Foo {
    Bar,
    Bing,
}

#[enumtrait::store(foo_trait_store)]
trait FooTrait {
    const baz: usize;
    fn foo(&self) -> usize;
}

impl FooTrait for Bar {
    const baz: usize = 2;  
    fn foo(&self) -> usize { self.bar_field }
}

impl FooTrait for Bing {  
    const baz: usize = 2;
    fn foo(&self) -> usize { self.bing_field }
}

#[enumtrait::impl_trait(foo_trait_store for foo_macro_store)]
impl FooTrait for Foo {
    const baz: usize = 42;
}

fn check(f: Foo) -> usize {
    f.foo()
}
```

## Performance
When comparing the cost of a virtual call, versus call through enums.
```bash
cargo bench
```
```text
call_cost               fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ call_with_blackhole                │               │               │               │         │
   ├─ Concrete                        │               │               │               │         │
   │  ├─ 1              0.162 ns      │ 0.184 ns      │ 0.163 ns      │ 0.163 ns      │ 100     │ 819200
   │  ├─ 16             14.71 ns      │ 15.43 ns      │ 15.35 ns      │ 15.17 ns      │ 100     │ 12800
   │  ╰─ 268435456      75.95 ms      │ 91.18 ms      │ 80.98 ms      │ 81.65 ms      │ 100     │ 100
   ├─ Double                          │               │               │               │         │
   │  ├─ 1              0.094 ns      │ 1.111 ns      │ 0.095 ns      │ 0.105 ns      │ 100     │ 819200
   │  ├─ 16             1.89 ns       │ 23.5 ns       │ 2.088 ns      │ 2.239 ns      │ 100     │ 102400
   │  ╰─ 268435456      78.77 ms      │ 129.3 ms      │ 99.86 ms      │ 99.96 ms      │ 100     │ 100
   ├─ ImplDyn                         │               │               │               │         │
   │  ├─ 1              0.788 ns      │ 1.345 ns      │ 0.793 ns      │ 0.805 ns      │ 100     │ 204800
   │  ├─ 16             16.25 ns      │ 16.37 ns      │ 16.29 ns      │ 16.3 ns       │ 100     │ 12800
   │  ╰─ 268435456      249.2 ms      │ 288.8 ms      │ 258.9 ms      │ 261 ms        │ 100     │ 100
   ├─ Single                          │               │               │               │         │
   │  ├─ 1              0.15 ns       │ 1.917 ns      │ 0.18 ns       │ 0.194 ns      │ 100     │ 819200
   │  ├─ 16             14.95 ns      │ 15.58 ns      │ 15.11 ns      │ 15.13 ns      │ 100     │ 12800
   │  ╰─ 268435456      78.88 ms      │ 91.74 ms      │ 83.3 ms       │ 83.98 ms      │ 100     │ 100
   ╰─ Sixteen                         │               │               │               │         │
      ├─ 1              0.417 ns      │ 0.544 ns      │ 0.419 ns      │ 0.425 ns      │ 100     │ 409600
      ├─ 16             22.77 ns      │ 40.55 ns      │ 23.29 ns      │ 23.42 ns      │ 100     │ 6400
      ╰─ 268435456      73.63 ms      │ 87.76 ms      │ 77.36 ms      │ 78.1 ms       │ 100     │ 100
```


## Inter-Macro Communication
Rust macro invocations are independent, and affected by incremental compilation.
- Change in token input means macro needs to be re-expanded
- Macros can be expanded in any order
- Macros are not eagrely expanded (with an exception for some [built in macros](https://github.com/rust-lang/rust/blob/1.54.0/RELEASES.md#language))

This is highly restrictive, solutions to avoid this include:
- Communication through shared data structures or files (cannot share tokens)
- Avoiding communication (verbose)
- not all macros are necessarily invokes, due to incremental compilation

There are proposals for sharing macro state, defining macro order, through a new interface. All have the 
core drawback of requiring language/compiler changes. 

I hope such a feature (e.g. crate local macro persistent state, message passing between macros, etc ) is implemented, but in the meantime, we have this.

*Rust macro invocations are independent.* However, macro definitions are ordered. We can 
use changing macro definitions to force an ordered invocation of other macros.

*See [Little Book of Rust macros > Callbacks](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html)*

We do this by building immutable token stores from `macro_rules!` definitions that reapply macros that read.
```rust
// we can define a basic macro we want to pass information to as
macro_rules! my_macro {
    ($($t:tt)*) => {
        stringify!($($t)*)
    }
}

// We then use the macro_store pattern (trademark pending😂) to store tokens 
// in macros. This can be made into a proc_macro that produces a macro_rules, 
// as is done for `enumtrait::store`
macro_rules! my_name {
    ($p:ident => $($t:tt)*) => {
        $p!( $($t)* Oliver Killane ) // storing a name
    }
}
macro_rules! my_passtime {
    ($p:ident => $($t:tt)*) => {
        $p!( $($t)* makes unecessarily complex macros ) 
    }
}


// reading from name into my_macro, with some extra tokens passable
let name = my_name!(my_macro => some extra data and ); 

// reading from two means applying (tokens get collected over `=>`)
let message = my_name!(my_passtime => my_macro =>); 

assert_eq!(name, "some extra data and Oliver Killane");
assert_eq!(message, "Oliver Killane makes unecessarily complex macros");
```
With that we can now pass tokens between macros in a [DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph).

Additional modification of the `store` is required to support accessing and 
exporting from modules, as well as the differing item and expression macro 
contexts (trailing `;` on macro invocation).

`enumtrait` passes information between macros using this method.

## Related Work
### [enum_dispatch](https://gitlab.com/antonok/enum_dispatch/)
Attempts to solve the same problem as `enumtrait`, but communicates between macro 
expansions using a shared hashmap.

This technique is [discussed here](https://gitlab.com/antonok/enum_dispatch/#registry-and-linkage).

The abuse of `macro_rules!` used by `enumtrait` is more verbose than `enum_dispatch`'s, however allows 
for identifiers with spans to be communicated between macros, allowing better error messages.

### [eagre](https://github.com/Emoun/eager)
Simulates eagre execution of macros generated by its own `eagre_macro_rules!` macro.
