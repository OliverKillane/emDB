use crate::plan::ir2::primitives::AnyInteger;

#[derive(Debug)]
pub enum IntBinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug)]
pub enum IntExpr<Ctx> {
    Ctx(Ctx),
    Const(AnyInteger),
    Neg(Box<IntExpr<Ctx>>),
    BinOp {
        op: IntBinOp,
        left: Box<IntExpr<Ctx>>,
        right: Box<IntExpr<Ctx>>,
    },
    If {
        cond: Box<BoolExpr<Ctx>>,
        then: Box<IntExpr<Ctx>>,
        otherwise: Box<IntExpr<Ctx>>,
    },
}

#[derive(Debug)]
pub enum BoolBinOp {
    And,
    Or,
    Xor,
}

#[derive(Debug)]
pub enum BoolExpr<Ctx> {
    Ctx(Ctx),
    Const(bool),
    Neg(Box<BoolExpr<Ctx>>),
    BinOp {
        op: BoolBinOp,
        left: Box<BoolExpr<Ctx>>,
        right: Box<BoolExpr<Ctx>>,
    },
}
