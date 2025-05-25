pub unsafe trait Truth {}
pub struct Bool<const EXPR: bool>;

unsafe impl Truth for Bool<true> {}
