//! Wrappers for glue functions. Useful for doing type erasure.
//!
//! It might make sense to move this down into the opendp crate, and use it for Function and
//! PrivacyRelation/StabilityRelation, but that can be done later if this works out.

use std::ops::Deref;
use std::rc::Rc;

/// A wrapper for some type-erasing glue. The typical use is to capture a closure that binds over
/// the concrete type of an inner value, and can downcast correctly to work with the value. It works
/// as a smart pointer, so you can call deref and call it as a function. It also implements `PartialEq`
/// (always returning true), so that you can have one as a struct field and use `#[derive(PartialEq)]`.
///
/// The easiest case is to have the inner value be a function pointer (`fn`), rather than a
/// closure (`dyn Fn`). That's because you can't pass in a bare trait object like you can
/// with the system smart pointers. If you need to use an actual closure, you'll need to wrap
/// it in an `Rc` and call the `Glue::new_rc`.
///
/// (AV Note: I tried to implement type aliases for different function arities, but the compiler
/// didn't seem to recognize it as a callable then.)
pub struct Glue<T: ?Sized>(Rc<T>);

impl<T> Glue<T> {
    pub fn new(val: T) -> Self {
        Self::new_rc(Rc::new(val))
    }
}

impl<T: ?Sized> Glue<T> {
    pub fn new_rc(val: Rc<T>) -> Self {
        Self(val)
    }
}

impl<T: ?Sized> Clone for Glue<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for Glue<T> {
    fn eq(&self, _other: &Self) -> bool { true }
}

impl<T: ?Sized> Deref for Glue<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_pointer() {
        struct Foo {
            val: i32,
            add_glue: Glue<fn(&Self, &Self) -> i32>,
        }
        impl Foo {
            pub fn new(val: i32) -> Self {
                Self { val, add_glue: Glue::new(|self_: &Foo, other: &Foo| self_.val + other.val) }
            }
            pub fn add(&self, other: &Self) -> i32 {
                (self.add_glue)(self, other)
            }
        }

        let f1 = Foo::new(1);
        let f2 = Foo::new(2);
        let r = f1.add(&f2);
        assert_eq!(r, 3);
    }

    #[test]
    fn test_closure() {
        struct Foo {
            val: i32,
            add_glue: Glue<dyn Fn(&Self, &Self) -> i32>,
        }
        impl Foo {
            pub fn new(val: i32) -> Self {
                Self { val, add_glue: Glue::new_rc(Rc::new(|self_: &Foo, other: &Foo| self_.val + other.val)) }
            }
            pub fn add(&self, other: &Self) -> i32 {
                (self.add_glue)(self, other)
            }
        }

        let f1 = Foo::new(1);
        let f2 = Foo::new(2);
        let r = f1.add(&f2);
        assert_eq!(r, 3);
    }
}