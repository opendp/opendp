mod apply;
pub use apply::*;

mod create;
pub use create::*;

mod select;
pub use select::*;

mod subset;
pub use subset::*;

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::core::Domain;
use crate::data::Column;
use crate::domains::type_name;
use crate::error::Fallible;

pub type DataFrame<K> = HashMap<K, Column>;

#[derive(PartialEq)]
pub struct DataFrameDomain<K: Hash + Eq> {
    pub _marker: PhantomData<fn() -> K>,
}
impl<K: Hash + Eq> Clone for DataFrameDomain<K> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<K: Hash + Eq> DataFrameDomain<K> {
    pub fn new() -> Self {
        DataFrameDomain {
            _marker: PhantomData,
        }
    }
}
impl<K: Hash + Eq> Domain for DataFrameDomain<K> {
    type Carrier = HashMap<K, Column>;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        Ok(true)
    }
}

impl<K: Hash + Eq> Debug for DataFrameDomain<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataFrameDomain({:?})", type_name!(K))
    }
}

impl<K: Hash + Eq> Default for DataFrameDomain<K> {
    fn default() -> Self {
        Self::new()
    }
}
