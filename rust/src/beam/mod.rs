use std::ffi::{c_char, c_void};
use std::marker::PhantomData;

use crate::core::{FfiResult, Function, IntoAnyTransformationFfiResultExt, StabilityMap, Transformation};
use crate::domains::AllDomain;
use crate::error::Fallible;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util;
use crate::ffi::util::Type;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::{CheckNull, Number};

pub enum Collection<T> {
    Internal(InternalCollectionImpl<T>),
    External(ExternalCollectionImpl<T>),
}
impl<T: 'static> Collection<T> {
    fn new_internal(vec: Vec<T>) -> Self {
        Self::Internal(InternalCollectionImpl(vec))
    }
    fn new_external(runtime: ExternalRuntime, data: *const c_void) -> Self {
        Self::External(ExternalCollectionImpl {
            runtime,
            data,
            _t: PhantomData,
        })
    }
    fn map<U: 'static, F: Fn(&T) -> Fallible<U> + 'static>(&self, f: F) -> Fallible<Collection<U>> {
        match self {
            Collection::Internal(int) => int.map(f),
            Collection::External(ext) => ext.map(f),
        }
    }
    pub fn take(self) -> Fallible<Vec<T>> {
        match self {
            Collection::Internal(int) => int.take(),
            Collection::External(ext) => ext.take(),
        }
    }
}
impl<T> CheckNull for Collection<T> {
    fn is_null(&self) -> bool {
        false
    }
}

pub trait CollectionImpl<T> {
    fn map<U: 'static, F: Fn(&T) -> Fallible<U> + 'static>(&self, f: F) -> Fallible<Collection<U>>;
    fn take(self) -> Fallible<Vec<T>>;
}

pub struct InternalCollectionImpl<T>(Vec<T>);
impl<T> CollectionImpl<T> for InternalCollectionImpl<T> {
    fn map<U: 'static, F: Fn(&T) -> Fallible<U> + 'static>(&self, f: F) -> Fallible<Collection<U>> {
        let res: Fallible<_> = self.0.iter().map(f).collect();
        res.map(Collection::new_internal)
    }
    fn take(self) -> Fallible<Vec<T>> {
        Ok(self.0)
    }
}

pub struct ExternalCollectionImpl<T> {
    runtime: ExternalRuntime,
    data: *const c_void,
    _t: PhantomData<T>,
}
impl<T: 'static> CollectionImpl<T> for ExternalCollectionImpl<T> {
    fn map<U: 'static, F: Fn(&T) -> Fallible<U> + 'static>(&self, f: F) -> Fallible<Collection<U>> {
        println!("external map");
        let closure = Closure1::new_fallible(f);
        // Have to put closure on on the heap?
        let closure = util::into_raw(closure);
        let T = Type::of::<T>().descriptor;
        let U = Type::of::<U>().descriptor;
        println!("calling runtime.map({:p}, {:p}...)", self.data, closure);
        let res = (self.runtime.map)(self.data, closure, util::into_c_char_p(T)?, util::into_c_char_p(U)?);
        // let _ = util::into_owned(closure);
        println!("RUST map method returned {:p}", res);
        let res = util::into_owned(res)?;
        println!("RUST map FFIResult into_owned done");
        match res {
            FfiResult::Ok(ptr) => {
                println!("Ok({:p})", ptr)
            }
            FfiResult::Err(err) => {
                println!("Err({:p})", err)
            }
        }
        let res: Fallible<_> = res.into();
        println!("RUST map FFIResult into done");
        let res = res.map(Downcast::downcast);
        println!("RUST map FFIResult downcast done {}", res.is_ok());
        let res = res?;
        println!("RUST map FFIResult into done");
        res
    }
    fn take(self) -> Fallible<Vec<T>> {
        let T = Type::of::<T>().descriptor;
        let res = (self.runtime.take)(self.data, util::into_c_char_p(T)?);
        let res: Fallible<_> = util::into_owned(res)?.into();
        res.map(Downcast::downcast)?
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExternalRuntime {
    // map: ExternalMethod3<Closure1, c_char, c_char, AnyObject>,
    // take: ExternalMethod1<c_char, AnyObject>,
    map: extern "C" fn(data: *const c_void, arg: *const Closure1, T: *const c_char, U: *const c_char) -> *mut FfiResult<*mut AnyObject>,
    take: extern "C" fn(data: *const c_void, T: *const c_char) -> *mut FfiResult<*mut AnyObject>,
}

// pub type ExternalMethod0<TO> = extern "C" fn(data: *const c_void) -> *mut FfiResult<*mut TO>;
// pub type ExternalMethod1<TI, TO> = extern "C" fn(data: *const c_void, arg: *const TI) -> *mut FfiResult<*mut TO>;
// pub type ExternalMethod2<TI0, TI1, TO> = extern "C" fn(data: *const c_void, arg0: *const TI0, arg1: *const TI1) -> *mut FfiResult<*mut TO>;
// pub type ExternalMethod3<TI0, TI1, TI2, TO> = extern "C" fn(data: *const c_void, arg0: *const TI0, arg1: *const TI1, arg2: *const TI2) -> *mut FfiResult<*mut TO>;

pub struct Closure1 {
    f: Box<dyn Fn(*const c_void, *mut c_void) -> Fallible<()>>,
}
impl Closure1 {
    pub fn new_fallible<TI, TO>(f: impl Fn(&TI) -> Fallible<TO> + 'static) -> Self {
        let f = move |argp: *const c_void, retp: *mut c_void| -> Fallible<()> {
            println!("RUST closure inner ({:p}, {:p})", argp, retp);
            let (argp, retp) = (argp as *const TI, retp as *mut TO);
            let arg = try_as_ref!(argp);
            let ret = f(arg)?;
            unsafe {
                *retp = ret;
            }
            Ok(())
        };
        Self { f: Box::new(f) }
    }
    pub fn new<TI, TO>(f: impl Fn(&TI) -> TO + 'static) -> Self {
        Self::new_fallible(move |arg| Ok(f(arg)))
    }
    pub fn call(&self, argp: *const c_void, retp: *mut c_void) -> Fallible<()> {
        println!("RUST closure call({:p}, {:p})", argp, retp);
        (self.f)(argp, retp)
    }
}

pub struct Closure2 {
    f: Box<dyn Fn(*const c_void, *const c_void, *mut c_void) -> Fallible<()>>,
}
impl Closure2 {
    pub fn new_fallible<TI0, TI1, TO>(f: impl Fn(&TI0, &TI1) -> Fallible<TO> + 'static) -> Self {
        let f = move |arg0p: *const c_void, arg1p: *const c_void, retp: *mut c_void| -> Fallible<()> {
            let (arg0p, arg1p, retp) = (arg0p as *const TI0, arg1p as *const TI1, retp as *mut TO);
            let (arg0, arg1) = (try_as_ref!(arg0p), try_as_ref!(arg1p));
            let ret = f(arg0, arg1)?;
            unsafe {
                *retp = ret;
            }
            Ok(())
        };
        Self { f: Box::new(f) }
    }
    pub fn new<TI0, TI1, TO>(f: impl Fn(&TI0, &TI1) -> TO + 'static) -> Self {
        Self::new_fallible(move |arg0, arg1| Ok(f(arg0, arg1)))
    }
    pub fn call(&self, arg0: *const c_void, arg1: *const c_void, ret: *mut c_void) -> Fallible<()> {
        (self.f)(arg0, arg1, ret)
    }
}

#[no_mangle]
pub extern "C" fn opendp_beam__call_closure_1(
    closure: *const Closure1,
    argp: *const c_void,
    retp: *mut c_void,
) -> FfiResult<*mut ()> {
    println!("RUST opendp_beam__call_closure_1 ({:p}, {:p}, {:p})", closure, argp, retp);
    let closure = try_as_ref!(closure);
    closure.call(argp, retp).into()
}

#[no_mangle]
pub extern "C" fn opendp_beam__new_collection_methods(
    map: extern "C" fn(data: *const c_void, arg0: *const Closure1, arg1: *const c_char, arg2: *const c_char) -> *mut FfiResult<*mut AnyObject>,
    take: extern "C" fn(data: *const c_void, arg: *const c_char) -> *mut FfiResult<*mut AnyObject>,
    data: *const c_void,
    T: *const c_char,
) -> FfiResult<*mut AnyObject> {
    let runtime = ExternalRuntime { map, take };
    let res = opendp_beam__new_collection(runtime, data, T);
    println!("Done new collection");
    res
}


#[no_mangle]
pub extern "C" fn opendp_beam__new_collection(
    runtime: ExternalRuntime,
    data: *const c_void,
    T: *const c_char,
) -> FfiResult<*mut AnyObject> {
    println!("new_collection");
    println!("new_collection, runtime={:?}, data={:?}, T={:p}", runtime, data, T);
    fn monomorphize<T: 'static>(
        runtime: ExternalRuntime,
        data: *const c_void,
    ) -> FfiResult<*mut AnyObject> {
        println!("new_collection, data={:?}, T={:?}, map={:p}", data, std::any::type_name::<T>(), runtime.map as *const c_void);
        let obj = AnyObject::new(Collection::<T>::new_external(runtime, data));
        let obj = util::into_raw(obj);
        println!("new_collection created {:p}", obj);
        // Ok(obj).into()
        FfiResult::Ok(obj)
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (runtime, data))
}

#[no_mangle]
pub extern "C" fn opendp_beam__get_data(
    obj: *mut AnyObject,
    T: *const c_char,
) -> FfiResult<*mut c_void> {
    fn monomorphize<T: 'static>(
        obj: *mut AnyObject,
    ) -> FfiResult<*mut c_void> {
        println!("get_data, obj={:p}", obj);
        let obj = try_!(util::into_owned(obj));
        let collection: Collection<T> = try_!(obj.downcast());
        match collection {
            Collection::Internal(_int) => err!(FFI, "Called get_data on Collection::Internal").into(),
            Collection::External(ext) => FfiResult::Ok(ext.data as *mut c_void),
        }
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (obj))
}

#[no_mangle]
pub extern "C" fn opendp_beam__make_mul(constant: *const c_void, T: *const c_char) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T: Number>(constant: *const c_void) -> FfiResult<*mut AnyTransformation> {
        // TODO: Use this pattern elswhere.
        let constant = *try_as_ref!(constant as *const T);
        make_mul::<T>(constant).into_any()
    }
    let T = try_!(Type::try_from(T));
    dispatch!(monomorphize, [(T, @numbers)], (constant))
}

pub fn make_mul<T: Number>(
    constant: T,
) -> Fallible<
    Transformation<
        AllDomain<Collection<T>>,
        AllDomain<Collection<T>>,
        SymmetricDistance,
        AbsoluteDistance<f64>,
    >,
> {
    Ok(Transformation::new(
        AllDomain::new(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Collection<T>| arg.map(move |e| {
            println!("map ({:?})", e);
            Ok(*e * constant)
        })),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new(|_d_in| 1.0),
    ))
}

#[cfg(test)]
pub mod test {
    use std::mem::MaybeUninit;
    use crate::error::ExplainUnwrap;
    use super::*;

    #[test]
    fn test_mul_internal() -> Fallible<()> {
        let mul2 = make_mul(2)?;
        let arg = Collection::new_internal(vec![1, 2, 3]);
        let res = mul2.invoke(&arg)?;
        assert_eq!(res.take()?, [2, 4, 6]);
        println!("✌️");
        Ok(())
    }

    fn make_test_collection_external<T: 'static>(vec: Vec<T>) -> Collection<T> {
        let data = util::into_raw(vec) as *const c_void;
        extern "C" fn test_runtime_map(data: *const c_void, f: *const Closure1, T: *const c_char, U: *const c_char) -> *mut FfiResult<*mut AnyObject> {
            fn monomorphize<T: 'static, U: 'static>(data: *const c_void, f: *const Closure1) -> FfiResult<*mut AnyObject> {
                let arg = util::as_ref(data as *const Vec<T>).unwrap_test();
                let res: Vec<U> = arg.into_iter().map(|x| {
                    let mut y = MaybeUninit::<U>::uninit();
                    let (xp, yp) = (x as *const T as *const c_void, y.as_mut_ptr() as *mut c_void);
                    opendp_beam__call_closure_1(f, xp, yp);
                    unsafe { y.assume_init() }
                }).collect();
                let res = make_test_collection_external(res);
                Ok(AnyObject::new(res)).into()
            }
            let T = Type::try_from(T).unwrap_test();
            let U = Type::try_from(U).unwrap_test();
            let res = dispatch!(monomorphize, [(T, @numbers), (U, @numbers)], (data, f));
            util::into_raw(res)
        }
        extern "C" fn test_runtime_take(data: *const c_void, T: *const c_char) -> *mut FfiResult<*mut AnyObject> {
            fn monomorphize<T: 'static>(data: *const c_void) -> FfiResult<*mut AnyObject> {
                let res = util::into_owned(data as *mut Vec<T>).unwrap_test();
                Ok(AnyObject::new(res)).into()
            }
            let T = Type::try_from(T).unwrap_test();
            let res = dispatch!(monomorphize, [(T, @numbers)], (data));
            util::into_raw(res)
        }
        let runtime = ExternalRuntime { map: test_runtime_map, take: test_runtime_take };
        let T = Type::of::<T>();
        let collection = opendp_beam__new_collection(runtime, data, util::into_c_char_p(T.descriptor).unwrap_test());
        let collection: Fallible<_> = collection.into();
        collection.map(Downcast::downcast).unwrap_test().unwrap_test()
    }

    #[test]
    fn test_mul_external() -> Fallible<()> {
        let mul2 = make_mul(2)?;
        let arg = make_test_collection_external(vec![1, 2, 3]);
        let res = mul2.invoke(&arg)?;
        let res = res.take()?;
        assert_eq!(res, [2, 4, 6]);
        println!("✌️");
        Ok(())
    }
}
