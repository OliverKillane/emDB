# Chew
## What is this?
A library for taking bit-sized & aligned values from buffers.

## Why is this?
For highly compressed storage, and for interaction with network, specialized hardware, 
the _normal_ bytes aligned & sized integer types are insufficient.

```
a: i27
b: u3
d: u78
```

## Goals
 - As little overhead as possible, ideally just copying bytes

```rust
type x = Integer<const BITS: u8>
fn offset<const BITS: usize>(&[])
```

Use compile time guarentees on the length of the buffer
```rust
Cursor<const MIN: usize, T>
```
Enforce sizes at compile time with `where [(); M - N]: ,` trick (negative sizes not valid)

```
Cursor<const MIN: usize>
fn access<const OFFSET: usize, const SIZE: usize>(x: &impl Cursor<SIZE>) 
```


