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

use crate::core::{Domain, Function, StabilityMap, Transformation};
use crate::data::{Column, IsVec};
use crate::domains::{AllDomain, MapDomain};
use crate::metrics::{SymmetricDistance, IntDistance};
use crate::traits::{Hashable, ExactIntCast};
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
pub struct SizedDataFrameDomain<TC: Hashable>
{
    pub categories_keys: HashMap<TC, Box<dyn CatVec>>,
    pub categories_counts: HashMap<TC, Vec<usize>>
}


impl<TC: Hashable> SizedDataFrameDomain<TC> {
    pub fn new<CA: 'static + Hashable>(categories_keys: HashMap<TC, Vec<CA>>, categories_counts: HashMap<TC, Vec<usize>>) -> Fallible<Self> {
        if categories_keys.len() == categories_counts.len() && !categories_keys.keys().all(|k| categories_counts.contains_key(k)) {
            return fallible!(FailedFunction, "Set of colunms with keys and counts must be indentical.")
        }
        if categories_keys.keys().into_iter().any(|k| categories_keys.get(k).unwrap().len() != categories_counts.get(k).unwrap().len()) {
            return fallible!(FailedFunction, "Numbers of keys and counts must be indentical.")
        }
        Ok(SizedDataFrameDomain {
            categories_keys: categories_keys.into_iter()
                .map(|(k, keys)| (k, Box::new(keys) as Box<dyn CatVec>)).collect(),
            categories_counts: categories_counts,
        })
    }
}

impl<TC: Hashable> SizedDataFrameDomain<TC> where TC: Eq + Hash + Clone {
    pub fn Default() -> Self  {
        SizedDataFrameDomain {
            categories_keys: HashMap::new(),
            categories_counts: HashMap::new(),
        }
    }
}

impl<TC: Hashable> SizedDataFrameDomain<TC> where TC: Clone {
    pub fn add_categorical_colunm<CA: 'static + Hashable>(&mut self, col_name: TC, column_categories: Vec<CA>, colummn_counts: Vec<usize>) -> Fallible<bool>  {
        if column_categories.len() != colummn_counts.len() {
            return fallible!(FailedFunction, "Colunm categories and counts must be indentical.")
        }
        self.categories_keys.insert(col_name.clone(), Box::new(column_categories) as Box<dyn CatVec>);
        self.categories_counts.insert(col_name.clone(), colummn_counts);
        Ok(true)
    }
}

impl<TC: Hashable> SizedDataFrameDomain<TC> where TC: Clone {
    pub fn create_categorical_df_domain<CA: 'static + Hashable>(col_names: Vec<TC>, column_categories: Vec<Vec<CA>>, colummn_counts: Vec<Vec<usize>>) -> Fallible<SizedDataFrameDomain<TC>>  {
        let df: SizedDataFrameDomain<TC> = SizedDataFrameDomain::Default();
        (0..(col_names.len())).map(|i|
        df.add_categorical_colunm(col_names.get(i).unwrap().clone(),
            column_categories.get(i).unwrap().clone(),
            colummn_counts.get(i).unwrap().clone()));
        Ok(df)
    }
}

impl<TC: Hashable> Domain for SizedDataFrameDomain<TC> {
    type Carrier = HashMap<TC, Column>;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        // To be impletemented later
        Ok(true)
    }
}

/// Vectors of keys encoding categorical variables (For membership function impl).
pub trait CatVec: IsVec {
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>>;
}

impl PartialEq for dyn CatVec {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other.as_any())
    }
 }

 impl Clone for Box<dyn CatVec> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
 }

impl<T: 'static + Hashable> CatVec for Vec<T>
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


/// Make a Transformation that takes  DF and define a SizedDataFrame with categorical columns.
/// 
/// # Arguments
/// * `categorical_column_names` - Names of column to be defined as categorical.
/// * `column_categories` - Vector containing the vectors of categories for each categorical column.
/// * `colummn_counts` - Vector containing vectors of counts for each categorical column.
/// * `null_partition` - Whether to include a trailing null partition for rows that were not in the `partition_keys`
/// 
/// # Generics
/// * `TC` - Type of column names.
/// * `CA` - Type of values in the identifier column.
pub fn make_add_categorical_column<TC: Hashable, CA: Hashable>(
    categorical_column_names: Vec<TC>,
    column_categories: Vec<Vec<CA>>,
    colummn_counts: Vec<Vec<usize>>,
    null_partition: bool,
) -> Fallible<
    Transformation<
    DataFrameDomain<TC>,
    SizedDataFrameDomain<TC>,
    SymmetricDistance,
    SymmetricDistance,
    >,
> {
    // Compute the number of paritions output
    let output_partitions = column_categories.len() + if null_partition { 1 } else { 0 };
    let d_output_partitions = IntDistance::exact_int_cast(output_partitions)?;

    // Create SizedDataFrameDomain with / without null partition
    let df_domain = SizedDataFrameDomain::create_categorical_df_domain(categorical_column_names,column_categories, colummn_counts).unwrap();
    
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        df_domain,
        Function::new_fallible(move |data: &DataFrame<TC>| {
            if null_partition == true {
                if df_domain.categories_keys.contains_key("None") {
                    // Add unknown category to categries
                    df_domain.categories_keys.iter().map(|(c,v)| v);
                }
                // Count unknowns in data
                // To be implemented
                // Add counts to unknown categories
                df_domain.categories_counts.iter().map(|(c,v)| v);
            }

            Ok(*data)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new(move |d_in: &IntDistance|  *d_in),
    ))
}