// TODO: Mark for each node the expression determining the size

// or
// unknown repeats
// reference
// sum

use smallvec::SmallVec;
use smart_arenas::prelude::*;

type ExprBitsKey<'id> = Key<'id, u16>;

enum ExprBits<'sizeexpr> {
    Repeated {
        known_times: Option<usize>,
        expr: ExprBitsKey<'sizeexpr>,
    },
    Choice {
        cases: SmallVec<[ExprBitsKey<'sizeexpr>; 2]>,
    },
    Exact(usize),
}

// impl <'id, A: Arena<'id, Data=ExprBits<'id>>> std::fmt::Debug for KeyWith<'_, 'id, A> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("KeyWith").field("key", &self.key).field("arena", &self.arena).finish()
//     }
// }
