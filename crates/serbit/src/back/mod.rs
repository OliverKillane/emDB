// Backend to produce docs
// Backend to produce nom parser
// Backend for native parse/update/serialize

/*

Data is &[u8]
Data is a &mut [u8]
Data is a stream of bytes (async)

let mut cursor = Cursor::new(&data);

let (next, stage1) = cursor.next()?;
stage1.a();
stage1.b();

let (next, stage2) next.next()?;
stage2.c();
stage2.zzz();

match next {
    OtherCase(next) => {
        let (next, stage) = next.next()?;
        stage.d();
        stage.e();

        let (next, stage) = next.next()?;
        stage.f();
        stage.g();

        let (next, stage) = next.next()?;
        stage.h();
        stage.i();
    },
    _ => {
    }

}

to write

let cursor = Cursor::new(&mut data);

to update
item.set_a();
item.set_b();

needs to be able to read integers (e.g. 3 bit).

Cost of borrow versus copy
 - determine if size > 8 bytes
 - if size ? 8 bytes, borrow

For values, place in arena

a;
until[b] { c };
repeat { d } until [e]{ f };

let (next, stage0) = next.next()?;
let (next, stage1) = next.next()?;
let (next, stage2) = next.next()?;



*/
