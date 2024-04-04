# Enumtrait
A crate for deriving enum based polymorphism.

## Pattern
Often we want to implement traits separately for each variant of a enum.
This is traditionally implemented in two ways:

```rust
trait Bing {
    fn bonk(&self);
}
 
struct Foo();
impl Bing for Foo {
    fn bonk(&self) {}
}
 
struct Bar();
impl Bing for Bar {
    fn bonk(&self) {}
}

fn method_1() {
    // using runtime polymorphism, at cost
    let bings: Vec<Box<dyn Bing>> = vec![Box::new(Foo()), Box::new(Bar())];
    for b in bings {
        b.bonk()
    }
}

fn method_2() {
    // using an enum, at the cost of boilerplate
    enum BingVars {
        Foo(Foo),
        Bar(Bar),
    }
    
    impl Bing for BingVars {
        fn bonk(&self) {
            match self {
                BingVars::Foo(i) => i.bonk(),
                BingVars::Bar(i) => i.bonk(),
            }
        }
    }

    let bings: Vec<BingVars> = vec![BingVars::Foo(Foo()), BingVars::Bar(Bar())];
    for b in bings {
        b.bonk()
    }
}

fn main() {
    method_1();
    method_2();
}
```

The crate removes the boilerplate from `method_2` by generating the enum and the implementation for you.
## Supported Types



## Related Work
### [enum_dispatch](https://gitlab.com/antonok/enum_dispatch/)
Attempts to solve the same problem as `enumtrait`, but communicates between macro 
expansions using a shared hashmap.

This technique is [discussed here](https://gitlab.com/antonok/enum_dispatch/#registry-and-linkage).

The abuse of `macro_rules!` used by `enumtrait` is more verbose than `enum_dispatch`'s, however allows 
for identifiers with spans to be communicated between macros, allowing better error messages.
