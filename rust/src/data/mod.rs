//! Framework for flexible abstract data type model for DataFrames.

#[cfg(feature = "ffi")]
mod ffi;

use crate::domains::type_name;
use crate::error::*;
use crate::traits::CheckNull;
use std::any::Any;
use std::fmt::Debug;

pub trait IsVec: Debug {
    // Not sure if we need into_any() (which consumes the Form), keeping it for now.
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn box_clone(&self) -> Box<dyn IsVec>;
    fn eq(&self, other: &dyn Any) -> bool;
    fn subset(&self, indicator: &Vec<bool>) -> Box<dyn IsVec>;
}

impl<T> IsVec for Vec<T>
where
    T: 'static + Debug + Clone + PartialEq,
{
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn box_clone(&self) -> Box<dyn IsVec> {
        Box::new(self.clone())
    }
    fn eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |o| o == self)
    }
    fn subset(&self, indicator: &Vec<bool>) -> Box<dyn IsVec> {
        Box::new(
            (self.iter())
                .zip(indicator)
                .filter_map(|(v, b)| b.then_some(v))
                .cloned()
                .collect::<Vec<_>>(),
        ) as Box<dyn IsVec>
    }
}

impl<T> From<Vec<T>> for Column
where
    T: 'static + Debug + Clone + PartialEq,
{
    fn from(src: Vec<T>) -> Self {
        Column::new(src)
    }
}

#[derive(Debug)]
pub struct Column(Box<dyn IsVec>);
impl CheckNull for Column {
    fn is_null(&self) -> bool {
        false
    }
}

impl Column {
    pub fn new<T: 'static + Debug>(form: Vec<T>) -> Self
    where
        Vec<T>: IsVec,
    {
        Column(Box::new(form))
    }
    pub fn as_form<T: 'static + IsVec>(&self) -> Fallible<&T> {
        self.0
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| err!(FailedCast, "tried to downcast to {:?}", type_name!(T)))
    }
    pub fn into_form<T: 'static + IsVec>(self) -> Fallible<T> {
        self.0
            .into_any()
            .downcast::<T>()
            .map_err(|_e| err!(FailedCast))
            .map(|v| *v)
    }
    pub fn subset(&self, indicator: &Vec<bool>) -> Self {
        Self(self.0.subset(indicator))
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
mod test;
