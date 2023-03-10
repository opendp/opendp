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
use std::fmt::{Debug};
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
/// TC: Categorical Colunm names
/// CA: Colunms Categories 
/// Counts: CaN Counts for Sized dataframe
/// # Example
#[derive(PartialEq, Clone, Debug)]
pub struct SizedDataFrameDomain<TC: Eq + Hash + Clone>
{
    pub categories_keys: HashMap<TC, Box<dyn IsVec>>,
    pub categories_counts: HashMap<TC, Vec<usize>>
}

impl<TC: Eq + Hash + Clone> SizedDataFrameDomain<TC> {
    pub fn new<CA: 'static + Eq + Hash + Debug + Clone>(categories_keys: HashMap<TC, Vec<CA>>, categories_counts: HashMap<TC, Vec<usize>>) -> Fallible<Self> {
        if categories_keys.len() == categories_counts.len() && !categories_keys.keys().all(|k| categories_counts.contains_key(k)) {
            return fallible!(FailedFunction, "Set of colunms with keys and counts must be indentical.")
        }
        if categories_keys.keys().into_iter().any(|k| categories_keys.get(k).unwrap().len() != categories_counts.get(k).unwrap().len()) {
            return fallible!(FailedFunction, "Numbers of keys and counts must be indentical.")
        }
        Ok(SizedDataFrameDomain {
            categories_keys: categories_keys.into_iter()
                .map(|(k, keys)| (k, Box::new(keys) as Box<dyn IsVec>)).collect(),
            categories_counts: categories_counts,
        })
    }
}

impl<TC: CheckNull> SizedDataFrameDomain<TC> where TC: Eq + Hash + Clone {
    pub fn Default() -> Self  {
        SizedDataFrameDomain {
            categories_keys: HashMap::new(),
            categories_counts: HashMap::new(),
        }
    }
}

impl<TC: Eq + Hash> SizedDataFrameDomain<TC> where TC: Clone {
    pub fn add_categorical_colunm<CA: 'static + Eq + Hash + Debug + Clone>(&mut self, col_name: TC, column_categories: Vec<CA>, colummn_counts: Vec<usize>) -> Fallible<bool>  {
        if column_categories.len() != colummn_counts.len() {
            return fallible!(FailedFunction, "Colunm categories and counts must be indentical.")
        }
        self.categories_keys.insert(col_name.clone(), Box::new(column_categories) as Box<dyn IsVec>);
        self.categories_counts.insert(col_name.clone(), colummn_counts);
        Ok(true)
    }
}

impl<TC: CheckNull + Debug + Clone> Domain for SizedDataFrameDomain<TC> where TC: Eq + Hash {
    type Carrier = HashMap<TC, Column>;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        // To be impletemented later
        Ok(true)
    }
}

/// A Type for Vectors of keys encoding categorical variables (For membership function impl).
pub trait CatVec: IsVec {
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>>;
}

impl<T: 'static + Eq + Hash> CatVec for Vec<T>
where
    Vec<T>: IsVec,
{
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>> {
        let cats = cats.as_any().downcast_ref::<Vec<T>>()?;
        let mut counts: HashMap<&T, usize> = cats.into_iter().map(|cat| (cat, 0)).collect();
        
        self.iter().try_for_each(|v| match counts.entry(v) {
            Entry::Occupied(v) => {
                *v.into_mut() += 1;
                Some(())
            },
            Entry::Vacant(_) => None,
        })?;

        Some(cats.iter().map(|c| counts.remove(c).unwrap()).collect())
    }
}