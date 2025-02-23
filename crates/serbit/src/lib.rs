//! ## Serbit
//! Basic idea:
//!  - A compiler to generate parser implementations for bit-specific protocols
//!
//! What do we need:
//!  - Ability to read & write
//!  - Ability to define data types of arbitrary size (e.g. bits)
//!  - Ability to hook into the backend, and use for other projects.
//!
//! Goals
//!  - To allow emdb to store data on disk, send over network, savouring every
//!    last bit.

/*

concepts:

 - struct
 - repeat
 - choice

we can also put assertions on the data (e.g. on alignment, on size)

 - Each stage takes some inputs, can then define how to parse.
 - Stages can have dependencies on previous
 - stages can have

struct || {
    member1: u8,
    member2: u34, @ size(<100) & align(4u8)
} -> || {
    member3: [s; member1]
    memberb: [s; varsize]
} -> || {
    member4: [s; member2]
}

Compiler can optimise:
 - If byte aligned, generate code with byte cursor
 - if possible, copy primitives directly
 - no recursion, but repeated fields allowed
 - bijective mapping only. Write/read can be generated and tested to be identical.
 - Arbitrary data generation

 , is same stage sequence
 ; is sequenced after

report warnings where unecesssary ; is used.

stage1[x: value]{
    j = u8, k = u8;
    if[j < 4]{
        inner = stage1; // boo cannot be recursive
    }
}

x = stage1;
for[x.value] {
    stage2;
}
stage3;
x = until[j.z > 4]{
    j = stage4[x.foo];
}
stage7;
stage: args, Vec<steps>. step: if, for, until, stage[args], properties
read or write, reference.
*/

/*
Struct{
    mem: Vec<FixedStage>
},
Sequence{
    steps: Vec<Stage>,
}
UntilPred{
    pred: BoolExpr,
    stage: Stage,
},
FixedRepeat{
    count: IntExpr,
    stage: FixedStage,
},
VarRepeat{
    count: IntExpr,
    stage: Stage,
},
Case{
    cases: Vec<(BoolExpr, Stage)>,
    otherwise: Option<Stage>,
},
FixedCase{
    cases: Vec<(Ident, BoolExpr, FixedStage)>
    otherwise: Option<(Ident, FixedStage)>,
},
*/
mod ir;
