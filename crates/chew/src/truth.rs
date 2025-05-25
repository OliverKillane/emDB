
pub unsafe trait Truth {}
pub struct Bool<const TRUTH: bool>;

unsafe impl Truth for Bool<true> {}


