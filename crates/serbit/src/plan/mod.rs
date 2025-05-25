pub mod ir;
pub mod pass;
pub mod namer;

/*
Supporting bit-specific sizes is hard

Reader struct needs to copy values out

struct {
    i12,
    u3,
    u23be,
    x: [u8; 23]
}

no copy, just pass:

struct {
    i: bit,
    i: i7,
    j: [X; 3]
}
*/

// Alignment requirements: for everything u8, except the smaller integers?
// cursor, methods to read/write, methods do the copying?

/*

struct {
    i2,
    i17,
}

cursor {
    contains methods to extract value, but does not extract
    just a ptr to the buffer
}

trait Cursor {
    type Value;
    type Read;

    fn at_#name()
    fn read_#name<field>(&self) -> Self::Read;
    fn write_#name<field>(&mut self, value: Self::Value);
}

how to extract value, not how to reference

struct {
    x: i2,
    j: i17,
}

let x = cursor(buffer)

x.read_x() -> i2

x.read_j() -> i17


for inner:


x.read_field() -> Cursor<Value = blagh>

we keep the value - why?

cursor.value() -> read all fields as struct

we can convert some values

cursor.read_aligned_string() -> &[u8];
cursor.read_aligned_x() -> &blagh

cursor.next()
cursor.resume()

additionally, returns are in terms of impl <Value> so as to not cause issues.

Cursor<Value> x;;;;

what do I need to generate:
 - semantic check
 - docs
 - generate the structs, and the cursor structs
 - generate the size constraints / positions for all

ByteSlice?
 - abstract above
 - we could operate on a stream
 - if we have bits?


struct Cursor<'brw> {
    buffer: &'brw mut [u8]
}

struct CursorMut<'brw> {
    buffer: &'brw mut [u8]
}


struct {
    x: i23, at pos (0)
    y: [bit; 5] at pos at (23) -> (0, 1, 2, 3, 4)
    z: i1 at (28)
}

size in bits.
size as an expression



*/
