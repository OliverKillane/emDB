pub type Bits = usize;
pub type Bytes = usize;

pub const fn bytes(bits: Bits) -> Bytes {
    (bits + 7) / 8
}
