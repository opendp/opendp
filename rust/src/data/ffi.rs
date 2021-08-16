use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::fmt::Formatter;
use std::hash::Hash;
use std::os::raw::c_char;
use std::slice;

use crate::{err, fallible, try_, try_as_ref};
use crate::core::{FfiError, FfiResult, FfiSlice};
use crate::data::Column;
use crate::error::Fallible;
use crate::ffi::any::{AnyObject, Downcast};
use crate::ffi::util;
use crate::ffi::util::{AnyMeasurementPtr, AnyTransformationPtr, c_bool, Type, TypeContents};

#[no_mangle]
pub extern "C" fn opendp_data__slice_as_object(raw: *const FfiSlice, T: *const c_char) -> FfiResult<*mut AnyObject> {
    let raw = try_as_ref!(raw);
    let T = try_!(Type::try_from(T));
    fn raw_to_plain<T: 'static + Clone>(raw: &FfiSlice) -> Fallible<AnyObject> {
        if raw.len != 1 {
            return fallible!(FFI, "The slice length must be one when creating a scalar from FfiSlice");
        }
        let plain = util::as_ref(raw.ptr as *const T)
            .ok_or_else(|| err!(FFI, "Attempted to follow a null pointer to create an object"))?.clone();
        Ok(AnyObject::new(plain))
    }
    fn raw_to_string(raw: &FfiSlice) -> Fallible<AnyObject> {
        let string = util::to_str(raw.ptr as *const c_char)?.to_owned();
        Ok(AnyObject::new(string))
    }
    fn raw_to_vec_string(raw: &FfiSlice) -> Fallible<AnyObject> {
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const *const c_char, raw.len) };
        let vec = slice.iter()
            .map(|str_ptr| Ok(util::to_str(*str_ptr)?.to_owned()))
            .collect::<Fallible<Vec<String>>>()?;
        Ok(AnyObject::new(vec))
    }
    fn raw_to_slice<T: Clone>(_raw: &FfiSlice) -> Fallible<AnyObject> {
        // TODO: Need to do some extra wrapping to own the slice here.
        unimplemented!()
    }
    #[allow(clippy::unnecessary_wraps)]
    fn raw_to_vec<T: 'static + Clone>(raw: &FfiSlice) -> Fallible<AnyObject> {
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const T, raw.len) };
        let vec = slice.to_vec();
        Ok(AnyObject::new(vec))
    }
    fn raw_to_tuple<T0: 'static + Clone, T1: 'static + Clone>(raw: &FfiSlice) -> Fallible<AnyObject> {
        if raw.len != 2 {
            return fallible!(FFI, "The slice length must be two when creating a tuple from FfiSlice");
        }
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const *const c_void, 2) };

        let tuple = util::as_ref(slice[0] as *const T0).cloned()
            .zip(util::as_ref(slice[1] as *const T1).cloned())
            .ok_or_else(|| err!(FFI, "Attempted to follow a null pointer to create a tuple"))?;
        Ok(AnyObject::new(tuple))
    }
    fn raw_to_hashmap<K: 'static + Clone + Hash + Eq, V: 'static + Clone>(raw: &FfiSlice) -> Fallible<AnyObject> {
        let slice = unsafe { slice::from_raw_parts(raw.ptr as *const *const AnyObject, raw.len) };

        // unpack keys and values into slices
        if slice.len() != 2 { return fallible!(FFI, "HashMap FfiSlice must have length 2"); }
        let keys = try_as_ref!(slice[0]).downcast_ref::<Vec<K>>()?;
        let vals = try_as_ref!(slice[1]).downcast_ref::<Vec<V>>()?;

        // construct the hashmap
        if keys.len() != vals.len() { return fallible!(FFI, "HashMap FfiSlice must have an equivalent number of keys and values"); };
        let map = keys.iter().cloned()
            .zip(vals.iter().cloned())
            .collect::<HashMap<K, V>>();
        Ok(AnyObject::new(map))
    }
    match T.contents {
        TypeContents::PLAIN("String") => {
            raw_to_string(raw)
        }
        TypeContents::SLICE(element_id) => {
            let element = try_!(Type::of_id(&element_id));
            dispatch!(raw_to_slice, [(element, @primitives)], (raw))
        }
        TypeContents::VEC(element_id) => {
            let element = try_!(Type::of_id(&element_id));
            println!("raw_to_hashmap: {:?}", element.descriptor);
            match element.descriptor.as_str() {
                "String" => raw_to_vec_string(raw),
                "AnyMeasurementPtr" => raw_to_vec::<AnyMeasurementPtr>(raw),
                "AnyObject" => raw_to_vec::<AnyObject>(raw),
                // "(AnyMeasurementPtr, f32)" => raw_to_vec::<(AnyMeasurementPtr, f32)>(raw),
                // "(AnyMeasurementPtr, f64)" => raw_to_vec::<AnyObjectPtr>(raw),
                "AnyTransformationPtr" => raw_to_vec::<AnyTransformationPtr>(raw),
                _ => dispatch!(raw_to_vec, [(element, @primitives)], (raw))
            }
        }
        TypeContents::TUPLE(ref element_ids) => {
            if element_ids.len() != 2 {
                return fallible!(FFI, "Only tuples of length 2 are supported").into();
            }
            let types = try_!(element_ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>());
            println!("from rust, dispatching types: {:?}", types);
            if types.first() == Some(&Type::of::<AnyMeasurementPtr>()) {
                dispatch!(raw_to_tuple, [(types[0], [AnyMeasurementPtr]), (types[1], [f64, f32])], (raw))
            } else {
                // In the inbound direction, we can handle tuples of primitives only. This is probably OK,
                // because the only likely way to get a tuple of AnyObjects is as the output of composition.
                dispatch!(raw_to_tuple, [(types[0], @primitives), (types[1], @primitives)], (raw))
            }
        }
        TypeContents::GENERIC { name, args } => {
            if name == "HashMap" {
                if args.len() != 2 { return err!(FFI, "HashMaps should have 2 type arguments").into(); }
                let K = try_!(Type::of_id(&args[0]));
                let V = try_!(Type::of_id(&args[1]));
                dispatch!(raw_to_hashmap, [(K, @hashable), (V, @primitives)], (raw))
            } else { fallible!(FFI, "unrecognized generic {:?}", name) }
        }
        // This list is explicit because it allows us to avoid including u32 in the @primitives
        _ => dispatch!(raw_to_plain, [(T, [u8, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, bool])], (raw))
    }.into()
}

#[no_mangle]
pub extern "C" fn opendp_data__object_type(this: *mut AnyObject) -> FfiResult<*mut c_char> {
    let obj: &AnyObject = try_as_ref!(this);

    match util::into_c_char_p(obj.type_.descriptor.to_string()) {
        Ok(v) => FfiResult::Ok(v),
        Err(e) => e.into()
    }
}

#[no_mangle]
pub extern "C" fn opendp_data__object_as_slice(obj: *const AnyObject) -> FfiResult<*mut FfiSlice> {
    let obj = try_as_ref!(obj);
    fn plain_to_raw<T: 'static>(obj: &AnyObject) -> Fallible<FfiSlice> {
        let plain: &T = obj.downcast_ref()?;
        Ok(FfiSlice::new(plain as *const T as *mut c_void, 1))
    }
    fn string_to_raw(obj: &AnyObject) -> Fallible<FfiSlice> {
        let string: &String = obj.downcast_ref()?;
        // FIXME: There's no way to get a CString without copying, so this leaks.
        Ok(FfiSlice::new(util::into_c_char_p(string.clone())? as *mut c_void, string.len() + 1))
    }
    fn vec_string_to_raw(obj: &AnyObject) -> Fallible<FfiSlice> {
        let vec_str: &Vec<String> = obj.downcast_ref()?;
        let vec = vec_str.iter()
            .cloned().map(util::into_c_char_p)
            .collect::<Fallible<Vec<*mut c_char>>>()?;

        let res = Ok(FfiSlice::new(vec.as_ptr() as *mut c_void, vec.len()));
        util::into_raw(vec);
        res
    }
    fn slice_to_raw<T>(_obj: &AnyObject) -> Fallible<FfiSlice> {
        // TODO: Need to get a reference to the slice here.
        unimplemented!()
    }
    fn vec_to_raw<T: 'static>(obj: &AnyObject) -> Fallible<FfiSlice> {
        let vec: &Vec<T> = obj.downcast_ref()?;
        Ok(FfiSlice::new(vec.as_ptr() as *mut c_void, vec.len()))
    }
    fn tuple_to_raw<T0: 'static, T1: 'static>(obj: &AnyObject) -> Fallible<FfiSlice> {
        let tuple: &(T0, T1) = obj.downcast_ref()?;
        Ok(FfiSlice::new(util::into_raw([
            &tuple.0 as *const T0 as *const c_void,
            &tuple.1 as *const T1 as *const c_void
        ]) as *mut c_void, 2))
    }
    fn hashmap_to_raw<K: 'static + Clone + Hash + Eq, V: 'static + Clone>(obj: &AnyObject) -> Fallible<FfiSlice> {
        let data: &HashMap<K, V> = obj.downcast_ref()?;

        // wrap keys and values up in an AnyObject
        let keys = AnyObject::new(data.keys().cloned().collect::<Vec<K>>());
        let vals = AnyObject::new(data.values().cloned().collect::<Vec<V>>());

        // wrap the whole map up together in an FfiSlice
        let map = vec![util::into_raw(keys), util::into_raw(vals)];
        let map_slice = FfiSlice::new(map.as_ptr() as *mut c_void, map.len());
        util::into_raw(map);
        Ok(map_slice)
    }
    match &obj.type_.contents {
        TypeContents::PLAIN("String") => {
            string_to_raw(obj)
        }
        TypeContents::SLICE(element_id) => {
            let element = try_!(Type::of_id(element_id));
            dispatch!(slice_to_raw, [(element, @primitives)], (obj))
        }
        TypeContents::VEC(element_id) => {
            let element = try_!(Type::of_id(element_id));
            if element.descriptor == "String" {
                vec_string_to_raw(obj)
            } else {
                dispatch!(vec_to_raw, [(element, @primitives)], (obj))
            }
        }
        TypeContents::TUPLE(element_ids) => {
            if element_ids.len() != 2 {
                return fallible!(FFI, "Only tuples of length 2 are supported").into();
            }
            let types = try_!(element_ids.iter().map(Type::of_id).collect::<Fallible<Vec<_>>>());
            // In the outbound direction, we can handle tuples of both primitives and AnyObjects.
            dispatch!(tuple_to_raw, [(types[0], @primitives_plus), (types[1], @primitives_plus)], (obj))
        }
        TypeContents::GENERIC { name, args } => {
            if name == &"HashMap" {
                if args.len() != 2 { return err!(FFI, "HashMaps should have 2 type arguments").into(); }
                let K = try_!(Type::of_id(&args[0]));
                let V = try_!(Type::of_id(&args[1]));
                dispatch!(hashmap_to_raw, [(K, @hashable), (V, @primitives)], (obj))
            } else { fallible!(FFI, "unrecognized generic {:?}", name) }
        }
        _ => { dispatch!(plain_to_raw, [(&obj.type_, @primitives)], (obj)) }
    }.into()
}

#[no_mangle]
pub extern "C" fn opendp_data__object_free(this: *mut AnyObject) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
/// Frees the slice, but not what the slice references!
pub extern "C" fn opendp_data__slice_free(this: *mut FfiSlice) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_data__str_free(this: *mut c_char) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}

#[no_mangle]
pub extern "C" fn opendp_data__bool_free(this: *mut c_bool) -> FfiResult<*mut ()> {
    util::into_owned(this).map(|_| ()).into()
}


impl std::fmt::Debug for AnyObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn monomorphize<T: 'static + std::fmt::Debug>(this: &AnyObject) -> Fallible<String> {
            Ok(match this.downcast_ref::<T>() {
                Ok(v) => format!("{:?}", v),
                Err(e) => e.to_string()
            })
        }
        let type_arg = &self.type_;
        f.write_str(dispatch!(monomorphize, [(type_arg, [
            u32, u64, i32, i64, f32, f64, bool, String, u8, Column,
            Vec<u32>, Vec<u64>, Vec<i32>, Vec<i64>, Vec<f32>, Vec<f64>, Vec<bool>, Vec<String>, Vec<u8>, Vec<Column>, Vec<Vec<String>>,
            HashMap<String, Column>,
            // FIXME: The following are for Python demo use of compositions. Need to figure this out!!!
            (Box<i32>, Box<f64>),
            (Box<i32>, Box<u32>),
            (Box<(Box<f64>, Box<f64>)>, Box<f64>),
            (AnyObject, AnyObject),
            AnyObject
        ])], (self)).unwrap_or("[Non-debuggable]".to_string()).as_str())
    }
}

#[no_mangle]
pub extern "C" fn opendp_data__to_string(this: *const AnyObject) -> FfiResult<*mut c_char> {
    util::into_c_char_p(format!("{:?}", try_as_ref!(this))).map_or_else(
        |e| FfiResult::Err(util::into_raw(FfiError::from(e))),
        FfiResult::Ok)
}

#[cfg(test)]
mod tests {
    use crate::error::*;
    use crate::error::ExplainUnwrap;
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_slice_as_object_number() -> Fallible<()> {
        let raw_ptr = util::into_raw(999) as *mut c_void;
        let raw_len = 1;
        let raw = util::into_raw(FfiSlice::new(raw_ptr, raw_len));
        let res = opendp_data__slice_as_object(raw, "i32".to_char_p());
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_slice_as_object_string() -> Fallible<()> {
        let data = "Hello".to_owned();
        let raw_ptr = util::into_c_char_p(data.clone()).unwrap_test() as *mut c_void;
        let raw_len = data.len() + 1;
        let raw = util::into_raw(FfiSlice::new(raw_ptr, raw_len));
        let res = opendp_data__slice_as_object(raw, "String".to_char_p());
        let res: String = Fallible::from(res)?.downcast()?;
        assert_eq!(res, "Hello");
        Ok(())
    }

    #[test]
    fn test_slice_as_object_vec() -> Fallible<()> {
        let data = vec![1, 2, 3];
        let raw_ptr = data.as_ptr() as *mut c_void;
        let raw_len = data.len();
        let raw = util::into_raw(FfiSlice::new(raw_ptr, raw_len));
        let res = opendp_data__slice_as_object(raw, "Vec<i32>".to_char_p());
        let res: Vec<i32> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_slice_as_object_tuple_numbers() -> Fallible<()> {
        let raw_ptr = util::into_raw((util::into_raw(999), util::into_raw(-999))) as *mut c_void;
        let raw_len = 2;
        let raw = util::into_raw(FfiSlice::new(raw_ptr, raw_len));
        let res = opendp_data__slice_as_object(raw, "(i32, i32)".to_char_p());
        let res: (i32, i32) = Fallible::from(res)?.downcast()?;
        assert_eq!(res, (999, -999));
        Ok(())
    }

    #[test]
    fn test_data_as_raw_number() -> Fallible<()> {
        let obj = AnyObject::new_raw(999);
        let res = opendp_data__object_as_slice(obj);
        let res = Fallible::from(res)?;
        assert_eq!(res.len, 1);
        assert_eq!(util::as_ref(res.ptr as *const i32).unwrap_test(), &999);
        Ok(())
    }

    #[test]
    fn test_data_as_raw_string() -> Fallible<()> {
        let obj = AnyObject::new_raw("Hello".to_owned());
        let res = opendp_data__object_as_slice(obj);
        let res = Fallible::from(res)?;
        assert_eq!(res.len, 6);
        assert_eq!(util::into_string(res.ptr as *mut c_char).unwrap_test(), "Hello");
        Ok(())
    }

    #[test]
    fn test_data_as_raw_vec() -> Fallible<()> {
        let obj = AnyObject::new_raw(vec![1, 2, 3]);
        let res = opendp_data__object_as_slice(obj);
        let res = Fallible::from(res)?;
        assert_eq!(res.len, 3);
        assert_eq!(util::as_ref(res.ptr as *const [i32; 3]).unwrap_test(), &[1, 2, 3]);
        Ok(())
    }

    #[test]
    fn test_data_as_raw_tuple_numbers() -> Fallible<()> {
        let obj = AnyObject::new_raw((999, -999));
        let res = opendp_data__object_as_slice(obj);
        let res = Fallible::from(res)?;
        assert_eq!(res.len, 2);
        let res_ptr = util::as_ref(res.ptr as *const [*mut i32; 2]).unwrap_test();
        assert_eq!((util::as_ref(res_ptr[0]).unwrap_test(), util::as_ref(res_ptr[1]).unwrap_test()), (&999, &-999));
        Ok(())
    }

    #[test]
    fn test_data_as_raw_tuple_objects() -> Fallible<()> {
        let obj = AnyObject::new_raw((AnyObject::new(999), AnyObject::new(999.0)));
        let res = opendp_data__object_as_slice(obj);
        let res = Fallible::from(res)?;
        assert_eq!(res.len, 2);
        let res_ptr = util::as_ref(res.ptr as *const [*mut AnyObject; 2]).unwrap_test();
        assert_eq!(
            (util::as_ref(res_ptr[0]).unwrap_test().downcast_ref()?, util::as_ref(res_ptr[1]).unwrap_test().downcast_ref()?),
            (&999, &999.0)
        );
        Ok(())
    }
}
