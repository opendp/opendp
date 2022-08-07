use std::{fmt::Formatter, collections::HashMap};

use crate::{data::Column, traits::TotalOrd, error::Fallible, ffi::any::Downcast};

use super::AnyObject;
use crate::err;


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
            (AnyObject, AnyObject),
            AnyObject
        ])], (self)).unwrap_or("[Non-debuggable]".to_string()).as_str())
    }
}

impl PartialEq for AnyObject {
    fn eq(&self, other: &Self) -> bool {
        fn monomorphize<T: 'static + PartialEq>(this: &AnyObject, other: &AnyObject) -> Fallible<bool> {
            Ok(this.downcast_ref::<T>()? == other.downcast_ref::<T>()?)
        }

        let type_arg = &self.type_;
        dispatch!(monomorphize, [(type_arg, @hashable)], (self, other)).unwrap_or(false)
    }
}

impl PartialOrd for AnyObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        fn monomorphize<T: 'static + PartialOrd>(this: &AnyObject, other: &AnyObject) -> Fallible<Option<std::cmp::Ordering>> {
            Ok(this.downcast_ref::<T>()?.partial_cmp(other.downcast_ref::<T>()?))
        }

        let type_arg = &self.type_;
        dispatch!(monomorphize, [(type_arg, @numbers)], (self, other)).unwrap_or(None)
    }
}

impl TotalOrd for AnyObject {
    fn total_cmp(&self, other: &Self) -> Fallible<std::cmp::Ordering> {
        fn monomorphize<T: 'static + TotalOrd>(this: &AnyObject, other: &AnyObject) -> Fallible<std::cmp::Ordering> {
            this.downcast_ref::<T>()?.total_cmp(other.downcast_ref::<T>()?)
        }

        let type_arg = &self.type_;
        // type list is explicit because (f32, f32), (f64, f64) are not in @numbers
        dispatch!(monomorphize, [(type_arg, [
            u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, f32, f64, (f32, f32), (f64, f64)
        ])], (self, other))
    }
}