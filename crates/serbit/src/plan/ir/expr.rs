use super::plan;

pub enum MathBinOp {
    Subtract,
    Multiply,
    Add,
}

pub enum Int<BoolKey, IntKey> {
    Const { value: isize, kind: plan::Integer },
    Bin(MathBinOp, IntKey, IntKey),
    Choice(BoolKey, IntKey, IntKey),
}

pub enum LogicalBinOp {
    And,
    Or,
}

pub enum Bool<BoolKey> {
    Const(bool),
    Not(BoolKey),
    Bin(LogicalBinOp, BoolKey, BoolKey),
}
