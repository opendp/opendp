use ndarray::{Array2, ScalarOperand, ArrayD};

use crate::{
    core::{AllDomain, Array2Domain, Function, StabilityMap, SymmetricDistance, Transformation},
    error::Fallible,
    traits::{Float, Hashable, Primitive},
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

// https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html#impl-DivAssign%3CA%3E
// TA must be qualified as a ScalarOperand for it to be able to divide row!
pub fn clamp_ball<TA: Float + ScalarOperand>(mut arr: Array2<TA>, bound: TA) -> Array2<TA> {
    // using a for loop
    // for mut row in arr.rows_mut().into_iter() {
    //     let norm = row.dot(&row).sqrt();
    //     row /= TA::one().max(norm / bound);
    // }

    // using an iterator
    arr.rows_mut()
        .into_iter()
        .for_each(|mut row| row /= TA::one().max(row.dot(&row).sqrt() / bound));

    arr
}

pub fn histogramdd(arr: Array2<usize>, category_lengths: Vec<usize>) -> Fallible<ArrayD<usize>> {
    if arr.ncols() != category_lengths.len() {
        return fallible!(FailedFunction, "category_lengths must have the same number of elements as arr has axes");
    }

    let counts = ArrayD::zeros(category_lengths);

    // write arr counts into `counts`
    

    Ok(counts)
}

pub fn make_select_array<K: Hashable, TOA: Primitive>(
    col_names: Vec<K>,
) -> Fallible<
    Transformation<
        DataFrameDomain<K>,
        Array2Domain<AllDomain<TOA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    Ok(Transformation::new(
        DataFrameDomain::new_all(),
        Array2Domain::new_all(),
        Function::new_fallible(move |arg: &DataFrame<K>| dataframe_to_array(&col_names, arg)),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
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
