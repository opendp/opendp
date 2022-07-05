use ndarray::Array2;

use crate::{
    error::Fallible,
    traits::{Hashable, Primitive}, core::{SymmetricDistance, Transformation, Array2Domain, AllDomain, Function, StabilityMap},
};

use super::{DataFrame, DataFrameDomain};

fn dataframe_to_array<K: Hashable, TOA: Primitive>(
    col_names: &Vec<K>,
    dataframe: &DataFrame<K>,
) -> Fallible<Array2<TOA>> {
    let vecs = col_names
        .iter()
        .map(|k| {
            dataframe
                .get(k)
                .ok_or_else(|| err!(FailedFunction, "column does not exist: {:?}", k))?
                .as_form::<Vec<TOA>>()
        })
        .collect::<Fallible<Vec<&Vec<TOA>>>>()?;

    let nrows = vecs[0].len();
    let flat = vecs.into_iter().flatten().cloned().collect::<Vec<TOA>>();

    Array2::from_shape_vec((nrows, col_names.len()), flat)
        .map_err(|e| err!(FailedFunction, "{:?}", e))
}


pub fn make_select_array<K: Hashable, TOA: Primitive>(col_names: Vec<K>) -> Fallible<Transformation<DataFrameDomain<K>, Array2Domain<AllDomain<TOA>>, SymmetricDistance, SymmetricDistance>> {
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        Array2Domain::new_all(),
        Function::new_fallible(move |arg: &DataFrame<K>| {
            dataframe_to_array(&col_names, arg)
        }),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1)
    ))
}


#[cfg(test)]
mod test {
    use crate::data::Column;

    use super::*;

    #[test]
    fn test_make_select_array() -> Fallible<()> {

        let trans = make_select_array(vec![1, 2, 3])?;

        let mut df = DataFrame::new();
        df.insert(1, Column::new(vec![1., 2., 3.]));
        df.insert(2, Column::new(vec![4., 5., 6.]));
        df.insert(3, Column::new(vec![7., 8., 9.]));

        let arr: Array2<f64> = trans.invoke(&df)?;

        println!("{:?}", arr);
        Ok(())
    }
}