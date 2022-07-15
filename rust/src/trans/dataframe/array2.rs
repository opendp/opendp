use ndarray::{Array1, Array2, ArrayD, ScalarOperand};

use crate::{
    core::{AllDomain, Array2Domain, VectorDomain, Function, StabilityMap, SymmetricDistance, Transformation},
    error::Fallible,
    traits::{Float, Hashable, Primitive}
};

use super::{DataFrame, DataFrameDomain};

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

// NEED TO FIX! Not quite sure how to make the type signatures match here
// Don't think that the domain should be <usize>
pub fn make_ravel_multi_index<TOA: Primitive>(
    category_lengths: Vec<usize>,
) -> Fallible<
    Transformation<
        Array2Domain<AllDomain<usize>>,
        VectorDomain<AllDomain<usize>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    Ok(Transformation::new(
        Array2Domain::new_all(),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Array2<usize>| ravel_multi_index(arg, category_lengths)),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1),
    ))
}

// collects counts based on input 2d array and category lengths into a 1-dimensional counts vector
pub fn ravel_multi_index(arr: Array2<usize>, category_lengths: Vec<usize>) -> Fallible<Vec<usize>> {
    if arr.ncols() != category_lengths.len() {
        return fallible!(FailedFunction, "category_lengths must have the same number of elements as arr has axes");
    }
    let mut max_index = 1;
    let offsets = category_lengths.iter().map(|len| {
        let old_max_index = max_index;
        max_index *= len;
        old_max_index
    }).collect::<Array1<usize>>();

    let mut flat:Vec<usize> = vec![0; max_index];
    arr.rows().into_iter().for_each(|row| flat[row.dot(&offsets).clamp(0, max_index)] += 1);
    return Ok(flat);
}


pub fn make_reshape(counts: Vec<usize>, category_lengths: Vec<usize>) -> Fallible<ArrayD<usize>> {
    let reshaped = ArrayD::from_shape_vec(category_lengths.clone(), counts);
    // NEED TO FIX! I'm trying to return a fallible, but it seems as though reshaped
    //  returns an <ArrayBase, Shapeerror> that isn't compatible with the current Fallible return type
    if reshaped.is_ok() {
        return Ok(reshaped);
    }
    return fallible!(FailedFunction, "mismatched dimensions");
}

#[cfg(test)]
mod test {
    use ndarray::arr2;
    #[test]
    fn test_make_ravel_multi_index() -> Fallible<()> {
        let arr = arr2(&[[1, 2],[0, 1], [2, 0]]);
        println!("{:?}", arr);
        let raveled = ravel_multi_index(arr.clone(), vec![3, 3]);
        let expected = vec![0, 0, 1, 1, 0, 0, 0, 1, 0];
        println!("{:?}", raveled);
        assert_eq!(raveled, Ok(expected));
        Ok(())
    }

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

    // NEED TO FIX! Trying to create a synthetic data pipeline, in the future this will
    // probably live in a function, but creating a test here as I build out the necessary
    // functions. 
    #[test]
    fn synthetic_data_pipeline() -> Fallible<()> {
        let category_lengths = vec![3, 3];
        let mut df = DataFrame::new();
        df.insert(1, Column::new(vec![0, 0, 1, 2]));
        df.insert(2, Column::new(vec![1, 1, 1, 2]));
        // QUESTION: the way make_select_array is currently implemented, 
        // it will converted a dataframe into a 2d array like so

        // DATAFRAME
        // 1  2
        // -  -
        // 0, 1
        // 0, 1
        // 1, 1
        // 2, 2

        // ARRAY
        // [[0, 0, 1, 2]
        //  [1, 1, 1, 2]]

        // This seems like it is transposing the data? 
        // Is this the way we want to have the data represented?
        
        // NEED TO FIX! not sure how to get the data to pass through correctly here
        let trans = make_select_array(df.keys().cloned().collect())?;
        let arr: Array2<f64> = trans.invoke(&df)?;
        // QUESTION? How do you chain multiple transformations together?
        let raveled = make_ravel_multi_index(arr, category_lengths);
        let categories = (0..10).collect();

        // QUESTION? I'm not sure where in the pipeline make_find fits. Once we have the 
        // raveled vector, where do we need to convert the columns of the dataframe to integer values?
        let counts = make_count_by_categories(categories)?;


        println!("{:?}", arr);
        Ok(())
    }
    
}
