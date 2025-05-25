# Smart Arenas
## What is this?
A library of arena fast data structures, with ownership semantics similar to start pointers.

## Why?
Building complex data structures (e.g. graphs), with many (potentially mutable) references in safe rust is awkward, and generally falls into two approches.

 - `Rc<RefCell<T>>`, using runtime checks to prove safety & can panic, prevents leaking & access always gives `&mut T`/`&T` unconditionally.
 - Generational arenas, using runtime checks for the ABA problem (also requires generation count in the key) & for precense on every access, but providing safety at compile time & potentially more performant (/application dependent) memory layout

In either case, we give something up at runtime.

With smart arenas we aim to move these checks to compile time, for arena data structures.

## How to solve the ABA problem at compile time?
To solve this we need 2 guarentees:
1. if a key exists, it's corresponding value exists in the arena
2. it is not possible to use a key, and another *instance* of an arena

To achieve (1.) requires making keys non-copyable.
 - any key copy needs to entier be impossible (an `Own` arena / similar to `Box`), or have a reference count (a `Shared` arena / similar to `Rc`/`Arc`)

To achieve (2.) requires passing a type to both the key, that can *only* be used by a specific instance of the arena.

We can do this with a type map - if more than one arena is instantiated with a token type, then panic. However this introduced runtime overhead, and makes unit testing hard (requires a static map).

So instead we use lifetimes as an identity, attached to a non-copyable token.
 - By passing the lifetime using a non-copyable token to the arena, inside a closure, we enforce only a single instance can have this `'id` lifetime
 - By making the key, token and arena invariant in this lifetime we enforce that these identities must exactly match. 

## Contributions
Use of identifier lifetimes was popularised by [GhostCell](https://plv.mpi-sws.org/rustbelt/ghostcell/). In fact, this is almost identical to the `BrandedVec` from the [ghostcell paper](https://plv.mpi-sws.org/rustbelt/ghostcell/paper.pdf).

## Potential Improvements
Add implementations for transformed iterators
 - allow re-using allocations
