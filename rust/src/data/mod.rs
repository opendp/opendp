//! Framework for flexible abstract data type model for DataFrames.

#[cfg(feature="ffi")]
mod ffi;

use std::any::Any;
use std::fmt::Debug;
use crate::error::*;
use crate::traits::CheckNull;

pub trait IsVec: Debug {
    // Not sure if we need into_any() (which consumes the Form), keeping it for now.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn IsVec>;
    fn eq(&self, other: &dyn Any) -> bool;
}

impl<T> IsVec for Vec<T> where
    T: 'static + Debug + Clone + PartialEq {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
    fn as_any(&self) -> &dyn Any { self }
    fn box_clone(&self) -> Box<dyn IsVec> { Box::new(self.clone()) }
    fn eq(&self, other: &dyn Any) -> bool { other.downcast_ref::<Self>().map_or(false, |o| o == self) }
}

impl<T> From<Vec<T>> for Column
    where T: 'static + Debug + Clone + PartialEq {
    fn from(src: Vec<T>) -> Self {
        Column::new(src)
    }
}


#[derive(Debug)]
pub struct Column(Box<dyn IsVec>);
impl CheckNull for Column {
    fn is_null(&self) -> bool { false }
}

impl Column {
    pub fn new<T: 'static + Debug>(form: Vec<T>) -> Self where Vec<T>: IsVec {
        Column(Box::new(form))
    }
    pub fn as_form<T: 'static + IsVec>(&self) -> Fallible<&T> {
        self.0.as_any().downcast_ref::<T>()
            .ok_or_else(|| err!(FailedCast))
    }
    pub fn into_form<T: 'static + IsVec>(self) -> Fallible<T> {
        self.0.into_any().downcast::<T>()
            .map_err(|_e| err!(FailedCast))
            .map(|v| *v)
    }
}

impl Clone for Column {
    fn clone(&self) -> Self {
        Column(self.0.box_clone())
    }
}

impl PartialEq for Column {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.0.as_any())
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_vec() {
        let form = vec![1, 2, 3];
        test_round_trip(form);
    }

    // #[test]
    // #[should_panic]
    // fn test_bogus() {
    //     let form = (Data::new(vec![1, 2, 3]), Data::new(99.9));
    //     let data = Data::new(form);
    //     let _retrieved: Vec<String> = data.into_form();
    // }

    fn test_round_trip<T: 'static + IsVec + PartialEq>(form: T) {
        let data = Column(form.box_clone());
        assert_eq!(&form, data.as_form().unwrap_test());
        assert_eq!(form, data.into_form().unwrap_test())
    }
}
