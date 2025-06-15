use crate::plan::ir::primitives::AnyInteger;

#[derive(Debug)]
pub enum IntBinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug)]
pub enum SizeExpr<Ctx> {
    Ctx(Ctx),
    Const(AnyInteger),
    Neg(Box<SizeExpr<Ctx>>),
    BinOp {
        op: IntBinOp,
        left: Box<SizeExpr<Ctx>>,
        right: Box<SizeExpr<Ctx>>,
    },
    If {
        cond: Box<BoolExpr<Ctx>>,
        then: Box<SizeExpr<Ctx>>,
        otherwise: Box<SizeExpr<Ctx>>,
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
