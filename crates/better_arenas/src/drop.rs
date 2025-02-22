use std::mem::{ManuallyDrop, forget};

pub trait CanDropWith<Arg> {
    fn drop(self, arg: Arg);
}

/// Allows objects to define their own drop that takes additional arguments.
///  - If [drop] is called outside of a panick, it will panic.
///  - [CanDropWith] must be implemented for the types that can be used to drop.
/// 
/// ### Why not a compile time panic on drop?
/// This would be ideal, however in const contexts, we cannot determine if we 
/// are in a unwind (panic) context. Hence if any panic anywhere is possible 
/// for the lifetime of such an object, it will fail compilation. 
/// 
/// This includes functions such as [ManuallyDrop::take], which is needed in 
/// [DropWith] internally.
/// 
/// ### Examples
/// ```
/// use rcarena::drop::{CanDropWith, DropWith};
///
/// struct Foo {
///     data: u32,
/// }
///
/// impl CanDropWith<&'static str> for Foo {
///     fn drop(self, arg: &'static str) {
///        println!("Dropping Foo with arg: {arg}");
///    }
/// }
///
/// let foo = DropWith::new(Foo { data: 42 });
/// foo.drop("Hello, world!");
/// ```
/// When not dropping, we get a runtime panic
/// ```should_panic
/// # use rcarena::drop::{CanDropWith, DropWith};
/// # struct Foo {
/// #     data: u32,
/// # }
/// # impl CanDropWith<&'static str> for Foo {
/// #     fn drop(self, arg: &'static str) {
/// #        println!("Dropping Foo with arg: {}", arg);
/// #    }
/// # }
/// let foo = DropWith::new(Foo { data: 42 });
/// ```
pub struct DropWith<D>(ManuallyDrop<D>);

impl<D> DropWith<D> {
    pub fn new(data: D) -> Self {
        Self(ManuallyDrop::new(data))
    }
}

impl<D> DropWith<D> {
    pub fn drop<Arg>(mut self, arg: Arg)
    where
        D: CanDropWith<Arg>,
    {
        let value = unsafe {
            let value = ManuallyDrop::<D>::take(&mut self.0);
            forget(self);
            value
        };
        value.drop(arg);
    }
}

impl<D> Drop for DropWith<D> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            // JUSTIFY: Panic at runtime when not in a panic itself.
            //           - Adding a const panic causes failures at compile time
            //             anywhere we might unwind. This causes issues when
            //             trying [ManuallyDrop::take], which can panic.
            //           -
            //           Hence we settle for second best, a panic at runtime.
            panic!(
                "Attempted to drop undroppable type: {}",
                std::any::type_name::<D>()
            );
        } else {
            // JUSTIFY: Dropping in panic.
            //           - Want to clean up resources in case of panic.
            //           - Reduce additional noise
            unsafe {
                ManuallyDrop::drop(&mut self.0);
            }
        }
    }
}
