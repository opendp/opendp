mod apply;
pub use apply::*;

mod create;
pub use create::*;

mod select;
pub use select::*;

mod subset;
pub use subset::*;

use std::any::Any;
use std::hash::Hash;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::core::Domain;
use crate::data::Column;
use crate::domains::{AllDomain, MapDomain};
use crate::traits::CheckNull;
use crate::error::Fallible;

pub type DataFrame<K> = HashMap<K, Column>;
pub type DataFrameDomain<K> = MapDomain<AllDomain<K>, AllDomain<Column>>;


/// A Domain that contains dataframes.
/// 
/// # Proof Definition
/// `DataFrameDomain(K)` consists of all datasets where 
/// colName_domain (colunms names) are elements of key_domain (of type DK) and
/// values are elements of value_domain (of type DV).
/// 
/// DC: Colunms domain
/// CoN: Colunms names
/// CaN: Colunms Categories 
/// Counts: CaN Counts for Sized dataframe
/// # Example
pub struct SizedDataFrameDomain<CoN: Eq + Hash> {
    pub columns_names_domain: AllDomain<CoN>,
    pub columns_names: CoN,
    pub column_categories: Option<HashMap<CoN, Box<dyn Any>>>,
    pub categories_counts: Option<Vec<(CoN, Vec<usize>)>>
}

impl<CoN: Eq + Hash> SizedDataFrameDomain<CoN> {
    pub fn new<CaN: Eq + Hash>(col_names: CoN, key_set: Option<(CoN, Vec<CaN>)>, counts: Option<Vec<usize>>) -> Fallible<Self> {
        if counts.is_some() && key_set.is_none() {
            fallible!(FailedFunction, "cannot define counts without a key set")
        }
        Ok(SizedDataFrameDomain {
            columns_names_domain: AllDomain::<CoN>::new(),
            columns_names: col_names,
            column_categories: key_set
                .map(|(k, keys)| (k, Box::new(keys) as Box<dyn Any>)),
                categories_counts: counts
        })
    }
}
impl<CoN: CheckNull> SizedDataFrameDomain<CoN> where CoN: Eq + Hash {
    pub fn new_all(col_names: CoN) -> Fallible<Self>  {
        Self::new(col_names ,None,None)
    }
}

 impl<CoN: Eq + Hash> Clone for SizedDataFrameDomain<CoN> {
     fn clone(&self) -> Self { Self::new(self.columns_names, self.column_categories, self.categories_counts) }
}

// Do we need to test equality of domain and col names?
 impl<CoN: Eq + Hash> PartialEq for SizedDataFrameDomain<CoN> {
     fn eq(&self, _other: &Self) -> bool { true }
}

 impl<CoN: Eq + Hash> Debug for SizedDataFrameDomain<CoN> {
     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
         write!(f, "SizedDataFrameDomain({:?})", self.columns_names_domain)
     }
 }

impl<CoN: CheckNull> Domain for SizedDataFrameDomain<CoN> where CoN: Eq + Hash {
    type Carrier = HashMap<CoN, AllDomain<Column>>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        for (k, v) in val {
            if !self.columns_names_domain.member(k)? {
                return Ok(false)
            }
        }
        Ok(true)
    }
}