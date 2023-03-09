mod apply;
pub use apply::*;

mod create;
pub use create::*;

mod select;
pub use select::*;

mod subset;
pub use subset::*;

use std::hash::Hash;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::collections::hash_map::Entry;

use crate::core::Domain;
use crate::data::{Column, IsVec};
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
/// NC: Categorical Colunm names
/// CA: Colunms Categories 
/// Counts: CaN Counts for Sized dataframe
/// # Example
#[derive(PartialEq, Clone)]
pub struct SizedDataFrameDomain<NC: Eq + Hash>
{
    pub categories_keys: HashMap<NC, Box<dyn IsVec>>,
    pub categories_counts: HashMap<NC, Vec<usize>>
}

impl PartialEq for dyn IsVec {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other.as_any())
    }
 }

 impl Clone for Box<dyn IsVec> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
 }

impl<NC: Eq + Hash> SizedDataFrameDomain<NC> {
    pub fn new<CA: Eq + Hash + Debug + Clone>(categories_keys: HashMap<NC, Vec<CA>>, categories_counts: HashMap<NC, Vec<usize>>) -> Fallible<Self> {
        if categories_keys.len() != categories_counts.len() {
            return fallible!(FailedFunction, "Number of dataframe colunms with keys and counts and keys must match.")
        }
        // ADD check that each columns have matching keys and count lengths.
        //if categories_keys.into_iter().any(|k, cat, counts| cat.len() != counts.len()) {

        //}
        Ok(SizedDataFrameDomain {
            categories_keys: categories_keys.into_iter()
                .map(|(k, keys)| (k, Box::new(keys) as Box<dyn IsVec>)).collect(),
            categories_counts: categories_counts,
        })
    }
}


impl<NC: CheckNull> SizedDataFrameDomain<NC> where NC: Eq + Hash {
    pub fn Default() -> Self  {
        SizedDataFrameDomain {
            categories_keys: HashMap::new(),
            categories_counts: HashMap::new(),
        }
    }
}

 impl<NC: Eq + Hash + Debug> Debug for SizedDataFrameDomain<NC> {
     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
         write!(f, "Categorical columns and associated colunms are ({:?})", (self.categories_keys, self.categories_counts))
     }
 }

impl<NC: CheckNull + Debug + Clone> Domain for SizedDataFrameDomain<NC> where NC: Eq + Hash {
    type Carrier = HashMap<NC, Column>;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        // To be impletemented later
        Ok(true)
    }
}

/// A Type for Vectors of keys encoding categorical variables (For membership function impl).
pub trait CatVec: IsVec {
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>>;
}

impl<T: Eq + Hash> CatVec for Vec<T>
where
    Vec<T>: IsVec,
{
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>> {
        let cats = cats.as_any().downcast_ref::<Vec<T>>()?;
        let counts: HashMap<&T, usize> = cats.into_iter().map(|cat| (cat, 0)).collect();
        
        self.iter().try_for_each(|v| match counts.entry(v) {
            Entry::Occupied(v) => {
                *v.get_mut() += 1;
                Some(())
            },
            Entry::Vacant(_) => None,
        })?;

        Some(cats.iter().map(|c| counts.remove(c).unwrap()).collect())
    }
}