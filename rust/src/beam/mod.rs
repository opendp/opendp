use std::any::Any;
use crate::core::{FfiResult, Function, StabilityMap, Transformation};
use crate::domains::AllDomain;
use crate::error::Fallible;
use crate::ffi::any::AnyObject;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::CheckNull;

pub struct PCollection {}
impl CheckNull for PCollection {
    fn is_null(&self) -> bool {
        false
    }
}
impl PCollection {
    pub fn map(function: impl Fn(&i32) -> i32 + 'static) {
    }
}


pub trait Collection<T> {
    fn map(&self, f: impl Fn(&T) -> T + 'static) -> Box<dyn Collection<T>>;
    fn reduce(&self, f: impl Fn(&T) -> T + 'static) -> Box<dyn Collection<T>>;
}

pub struct VecCollection<T>(Vec<T>);
impl<T> Collection<T> for VecCollection<T> {
    fn map(&self, f: impl Fn(&T) -> T + 'static) -> Box<dyn Collection<T>> {
        self.0.into_iter().map(f).collect()
    }

    fn reduce(&self, function: impl Fn(&T) -> T + 'static) -> Box<dyn Collection<T>> {
        todo!()
    }
}


pub type MapFn = extern "C" fn(*const AnyObject, *const AnyObject) -> *mut FfiResult<*mut AnyObject>;
pub type MapImplFn = extern "C" fn(*const PCollection, MapFn, *const AnyObject) -> *mut FfiResult<*mut PCollection>;

fn mul_map_fn(x: *const AnyObject, ctx: *const AnyObject) -> *mut FfiResult<*mut AnyObject> {
    todo!()
}

fn local_map_impl_fn(arg: *const PCollection, f: MapFn, ctx: *const AnyObject) -> *mut FfiResult<*mut PCollection> {
    todo!()
}

pub fn make_mul_beam(x: i32, map_impl: MapImplFn) -> Fallible<
    Transformation<
        AllDomain<PCollection>,
        AllDomain<f64>,
        SymmetricDistance,
        AbsoluteDistance<f64>,
    >,
> {
    Ok(Transformation::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new(|arg| {
            map_impl(arg, mul_map_fn, std::ptr::null())
        }),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new(|_d_in| 1.0),
    ))
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_sum() -> Fallible<()> {
        let mul2 = make_mul_beam(2, local_map_impl_fn)?;
        let arg = PCollection {};
        let res = mul2.invoke(&arg)?;
        assert_eq!(res, 99.9);
        Ok(())
    }
}
