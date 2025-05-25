// use std::marker::PhantomData;
// /* Library: */
// trait ReadPrimitive<T> {
//     fn read<const BITS: usize>(&self) -> T;
// }

// trait WritePrimitive<T> {
//     fn write<const BITS: usize>(&mut self, value: T);
// }

// trait Cursor:
//     ReadPrimitive<u8>
//     + ReadPrimitive<u16>
//     + ReadPrimitive<u32>
//     + ReadPrimitive<u64>
//     + ReadPrimitive<u128>
//     + ReadPrimitive<i8>
//     + ReadPrimitive<i16>
//     + ReadPrimitive<i32>
//     + ReadPrimitive<i64>
//     + ReadPrimitive<i128>
//     + ReadPrimitive<bool>
//     + ReadPrimitive<char>
// {
//     fn advance<const BITS: usize>(&self) -> Self;
// }

// trait MutCursor:
//     Cursor
//     + WritePrimitive<u8>
//     + WritePrimitive<u16>
//     + WritePrimitive<u32>
//     + WritePrimitive<u64>
//     + WritePrimitive<u128>
//     + WritePrimitive<i8>
//     + WritePrimitive<i16>
//     + WritePrimitive<i32>
//     + WritePrimitive<i64>
//     + WritePrimitive<i128>
//     + WritePrimitive<bool>
//     + WritePrimitive<char>
// {
// }

// struct Writer<'brw, M: MutCursor, V> {
//     cursor: &'brw mut M,
//     _phantom: PhantomData<V>,
// }
// struct Reader<'brw, C: Cursor, V> {
//     cursor: &'brw C,
//     _phantom: PhantomData<V>,
// }

// /* Generated: */
// pub struct Struct1 {
//     pub x: i32,
//     pub y: u8,
// }

// impl<'brw, C: Cursor> Reader<'brw, C, Struct1> {
//     pub fn x(&self) -> i32 {
//         todo!()
//     }
//     pub fn y(&self) -> u8 {
//         todo!()
//     }
// }

// impl<'brw, M: MutCursor> Writer<'brw, M, Struct1> {
//     pub fn x(&mut self, value: i32) {
//         todo!()
//     }

//     pub fn y(&mut self, value: u8) {
//         todo!()
//     }
// }

// fn parse_what_i_want(reader: Reader<'_, impl Cursor, Struct1>) {
//     let z = reader.x();
// }

// /*
// most of the code just advances the cursor for next stage
// or for inner value, then create reader (wrapper on cursor)

// inlining helps + const params to give compiler info required to produce simple field access

// basic cursor on buffer:
//  - need to be able to load for buffer
//  - need to be able to resume the cursor, at a different buffer

// reader needs context
//  - for the next, and for the previous


// Reader<Item>
// Reader<Series> // implements next, so has a context type
// */

// /*
// fn foo(read: Reader<impl Cursot, MyStruct>) {
//     let y: Reader<_, MyStruct2> = read.x();
//     let my_3 = read.next();
//     my_3.z();
// }
// */

// // Item {}
// // Stage
// // Seq

// /*

// Can create an item easily
//  - reader
//  - reader/modifier
//  - to value / from value

// Seq -> 
//  - 

// Stage
//  - implements next
//  - creates items with context
//  */

// fn stage_foo() {
//     let begin = todo!();


//     let stage_1 = begin.start();
//     let (item, stage_2) = stage_1.next();
//     let (inner /* Iterator over stages */, stage_3) = stage_2.next();
//     let (item, stage_4) = stage_3.next();

//     // for until:
//     //  - generates an iterator / array over the stages

//     // single value
//     // repeat by n (previous stage)
//     // repeat until sentinel condition (e.g. strings) - 


//     do {
//         end: bit,
//         val: u7,
//     } until (!bit) include;


//     // so a string is
//     do {
//         char: ascii,
//     } until (ascii = '\0');


// }


