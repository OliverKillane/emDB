use std::marker::PhantomData;
struct Bob<D>(D);
struct BobWindow<'imm, D>(&'imm mut Bob<D>);

impl<D> Bob<D> {
    fn window(&mut self) -> BobWindow<'_, D> {
        BobWindow(self)
    }
}

struct Key<'a, D>(PhantomData<&'a D>);

impl<'imm, D> BobWindow<'imm, D> {
    fn get_key(&'a self) -> Key<'imm, D> {
        Key(PhantomData)
    }

    fn take_key<'a>(&'a self, key: Key<'imm, D>) {}
}

fn main() {
    let bool_bob = Bob(true);
    let bool_bob2 = Bob(true);
    let int_bob2 = Bob(23);

    bool_bob.take_key(bool_bob.get_key());
    bool_bob.take_key(bool_bob2.get_key());
}
