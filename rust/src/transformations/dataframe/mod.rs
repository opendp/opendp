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
use crate::traits::{Hashable};
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
    pub categories_keys: HashMap<TC, Box<dyn IsVec>>,
    pub categories_counts: HashMap<TC, Vec<usize>>
}


impl<TC: Hashable> SizedDataFrameDomain<TC> {
    pub fn new(categories_keys: HashMap<TC, Box<dyn IsVec>>, categories_counts: HashMap<TC, Vec<usize>>) -> Fallible<Self> {
        if categories_keys.len() == categories_counts.len() && !categories_keys.keys().all(|k| categories_counts.contains_key(k)) {
            return fallible!(FailedFunction, "Set of colunms with keys and counts must be indentical.")
        }
        // if categories_counts.keys().into_iter().any(|k| categories_counts.get(k).unwrap().len() != categories_counts.get(k).unwrap().len()) {
        //     return fallible!(FailedFunction, "Numbers of keys and counts must be indentical.")
        // }
        Ok(SizedDataFrameDomain {
            categories_keys: categories_keys,
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
        self.categories_keys.insert(col_name.clone(), Box::new(column_categories) as Box<dyn IsVec>);
        self.categories_counts.insert(col_name.clone(), colummn_counts);
        Ok(true)
    }
}

impl<TC: Hashable> SizedDataFrameDomain<TC> where TC: Clone {
    pub fn create_categorical_df_domain<CA: 'static + Hashable>(cat_col_name: TC, column_categories: Vec<CA>, colummn_counts: Vec<usize>) -> Fallible<SizedDataFrameDomain<TC>>  {
        let mut df: SizedDataFrameDomain<TC> = SizedDataFrameDomain::Default();
        df.add_categorical_colunm(cat_col_name, column_categories, colummn_counts).unwrap();
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
    fn box_clone(&self) -> Box<dyn CatVec>;
    fn counts(&self, cats: &dyn CatVec) -> Option<Vec<usize>>;
}

impl PartialEq for dyn CatVec {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other.as_any())
    }
 }

//  impl Clone for Box<dyn CatVec> {
//     fn clone(&self) -> Box<dyn CatVec> {
//         CatVec::box_clone(&self)
//     }
//  }

impl<T: 'static + Hashable> CatVec for Vec<T>
where
    Vec<T>: IsVec,
{
    fn box_clone(&self) -> Box<dyn CatVec> { Box::new(self.clone()) }
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
pub fn make_add_categorical_column_with_counts<TC: Hashable, CA: Hashable>(
    categorical_column_names: TC,
    column_categories: Vec<CA>,
    colummn_counts: Vec<usize>,
) -> Fallible<
    Transformation<
    DataFrameDomain<TC>,
    SizedDataFrameDomain<TC>,
    SymmetricDistance,
    SymmetricDistance,
    >,
> {
    // Create SizedDataFrameDomain with / without null partition
    let df_domain = SizedDataFrameDomain::create_categorical_df_domain(categorical_column_names,column_categories, colummn_counts).unwrap();
    
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        df_domain,
        Function::new_fallible(move |data: &DataFrame<TC>| {
            Ok(data.clone())
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new(move |d_in: &IntDistance|  *d_in),
    ))
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dataFrameDomain() -> Fallible<()> {
        let categorical_colNames:Vec<&str> = vec!["colA","colB"];
        let categories_colA: Vec<usize> = vec![1, 2, 3, 4];
        let categories_colB: Vec<&str> =  vec!["A", "B", "C"];

        let counts_colA: Vec<usize> = vec![223, 234, 4525, 34];
        let counts_colB: Vec<usize> = vec![218, 4229, 569];

        let mut col_categories = HashMap::new();
        col_categories.insert(categorical_colNames[0], Box::new(categories_colA.clone()) as Box<dyn IsVec>);
        col_categories.insert(categorical_colNames[1], Box::new(categories_colB.clone()) as Box<dyn IsVec>);

        let mut col_counts = HashMap::new();
        col_counts.insert(categorical_colNames[0], counts_colA.clone());
        col_counts.insert(categorical_colNames[1], counts_colB.clone());

        assert_eq!(SizedDataFrameDomain::new(col_categories, col_counts).is_err(), false);
       
        let mut inputDomain = SizedDataFrameDomain::Default();
        inputDomain.add_categorical_colunm(categorical_colNames[0], categories_colA.clone(), counts_colA.clone()).unwrap();
        inputDomain.add_categorical_colunm(categorical_colNames[1], categories_colB, counts_colB).unwrap();

        assert_eq!(SizedDataFrameDomain::create_categorical_df_domain(categorical_colNames[0], categories_colA, counts_colA).is_err(), false);
    
        Ok(())
    }
}