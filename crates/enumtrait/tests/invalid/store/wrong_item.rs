// enum Empty{}

// #[enumtrait::store(fn_store)]
// fn cool() {}

// #[enumtrait::store(trait_store)]
// trait Cool {}

// #[enumtrait::impl_trait(fn_store for trait_store)]
// impl Cool for Empty {} 

/*
TODO: Currently failing because of a compiler bug on nightly, needs to be fixed 
      so that the actual error message can be propagated. 

error: proc macro panicked
 --> tests/invalid/store/wrong_item.rs:6:1
  |
6 | #[enumtrait::store(trait_store)]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
9 | #[enumtrait::impl_trait(trait_store for fn_store)]
  | -------------------------------------------------- in this procedural macro expansion
  |
  = help: message: assertion failed: child.level.can_be_top_or_sub().1
  = note: this error originates in the macro `trait_store` which comes from the expansion of the attribute macro `enumtrait::impl_trait` (in Nightly builds, run with -Z macro-backtrace for more info)
*/ 

fn main() {  dont_compile_me }
