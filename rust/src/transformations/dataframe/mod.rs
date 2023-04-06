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

use polars::prelude::*;

use crate::core::{Domain, Function, StabilityMap, Transformation};
use crate::data::{Column, IsVec};
use crate::domains::{AllDomain, MapDomain};
use crate::metrics::{SymmetricDistance, IntDistance};
use crate::traits::{Hashable};
use crate::error::Fallible;

pub type DataFrame<K> = HashMap<K, Column>;
pub type DataFrameDomain<K> = MapDomain<AllDomain<K>, AllDomain<Column>>;

pub type SizedDataFrame = polars::prelude::DataFrame;

/// A Domain that contains dataframes.
/// 
/// # Proof Definition
/// `SizedDataFrameDomain(TC)` consists of all datasets where
/// `categories_keys` contains the columns names of categorical variable with their categories (keys) and
/// `categories_counts`contains the columns names of categorical variable with cateogries counts.
/// 
/// `TC`: Categorical Colunm names type
#[derive(PartialEq, Clone, Debug)]
pub struct SizedDataFrameDomain
{
    //pub categories_keys: HashMap<&'static str, polars::prelude::Series>,
    //pub categories_counts: HashMap<&'static str, Vec<usize>>
    pub categories_keys: Vec<polars::prelude::Series>,
    pub categories_counts: HashMap<&'static str, Vec<usize>>
}


impl SizedDataFrameDomain {
    pub fn new(categories_keys: Vec<polars::prelude::Series>, categories_counts: HashMap<&'static str, Vec<usize>>) -> Fallible<Self> {
        if categories_keys.len() == categories_counts.len() && !categories_keys.iter().map(|s| categories_counts.contains_key(s.name())).all(|k| k) {
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

impl SizedDataFrameDomain {
    pub fn Default() -> Self  {
        SizedDataFrameDomain {
            categories_keys: Vec::<polars::prelude::Series>::new(),
            categories_counts: HashMap::<&'static str, Vec<usize>>::new(),
        }
    }
}

impl SizedDataFrameDomain {
    pub fn add_categorical_colunm(&mut self, column_categories: &polars::prelude::Series, colummn_counts: Vec<usize>) -> Fallible<bool>  {
        if column_categories.len() != colummn_counts.len() {
            return fallible!(FailedFunction, "Colunm categories and counts must be indentical.")
        }
        // To be checked if that is a correct way to proceed!!!
        let col_name = Box::leak(column_categories.name().to_string().into_boxed_str());
        self.categories_keys.push(column_categories.clone());
        self.categories_counts.insert(col_name, colummn_counts);
        Ok(true)
    }
}

impl SizedDataFrameDomain {
    pub fn create_categorical_df_domain(column_categories: &polars::prelude::Series, colummn_counts: Vec<usize>) -> Fallible<SizedDataFrameDomain>  {
        let mut df: SizedDataFrameDomain = SizedDataFrameDomain::Default();
        df.add_categorical_colunm(column_categories, colummn_counts).unwrap();
        Ok(df)
    }
}

impl Domain for SizedDataFrameDomain {
    //type Carrier = HashMap<TC, Column>;
    type Carrier = polars::prelude::DataFrame;
    fn member(&self, _val: &Self::Carrier) -> Fallible<bool> {
        // TO BE IMPLEMENTED.
        Ok(true)
    }
}

/// Vector type for categorical variables.
pub trait CatVec: IsVec {
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
// pub fn make_add_categorical_column_with_counts<TC: Hashable, CA: Hashable>(
//     categorical_column_names: TC,
//     column_categories: Vec<CA>,
//     colummn_counts: Vec<usize>,
// ) -> Fallible<
//     Transformation<
//     DataFrameDomain<TC>,
//     SizedDataFrameDomain<TC>,
//     SymmetricDistance,
//     SymmetricDistance,
//     >,
// > {
//     // Create SizedDataFrameDomain with / without null partition
//     let df_domain = SizedDataFrameDomain::create_categorical_df_domain(categorical_column_names,column_categories, colummn_counts).unwrap();
    
//     Ok(Transformation::new(
//         DataFrameDomain::new_all(),
//         df_domain,
//         Function::new_fallible(move |data: &SizedDataFrame| {
//             // No-op function but request for now cloning data. Might be possible to modify to work on pointer only.
//             Ok(data.clone())
//         }),
//         SymmetricDistance::default(),
//         SymmetricDistance::default(),
//         StabilityMap::new(move |d_in: &IntDistance|  *d_in),
//     ))
// }


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dataFrameDomain() -> Fallible<()> {
        let categorical_colNames:Vec<&str> = vec!["colA","colB"];
        
        let categorical_colA = polars::prelude::Series::new(categorical_colNames[0], [1, 2, 3, 4]);
        let categorical_colB = polars::prelude::Series::new(categorical_colNames[1], ["A", "B", "C"]);
       
       
        let counts_colA: Vec<usize> = vec![223, 234, 4525, 34];
        let counts_colB: Vec<usize> = vec![218, 4229, 569];

        let mut col_categories = Vec::<polars::prelude::Series>::new();
        col_categories.push(categorical_colA.clone());
        col_categories.push(categorical_colB.clone());

        let mut col_counts = HashMap::new();
        col_counts.insert(categorical_colNames[0], counts_colA.clone());
        col_counts.insert(categorical_colNames[1], counts_colB.clone());

        assert_eq!(SizedDataFrameDomain::new(col_categories, col_counts).is_err(), false);
       
        //let domain_test = SizedDataFrameDomain::new(col_categories, col_counts);
        //println!("{:?}", domain_test);

        let mut inputDomain = SizedDataFrameDomain::Default();
        inputDomain.add_categorical_colunm(&categorical_colA, counts_colA.clone()).unwrap();
        inputDomain.add_categorical_colunm(&categorical_colB, counts_colB).unwrap();

        //println!("{:?}", inputDomain);

        assert_eq!(SizedDataFrameDomain::create_categorical_df_domain(&categorical_colA, counts_colA).is_err(), false);
    
        Ok(())
    }
}