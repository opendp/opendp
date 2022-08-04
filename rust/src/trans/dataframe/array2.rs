use ndarray::{Array1, Array2, ArrayD, Dimension};
use super::{DataFrame, DataFrameDomain};

use crate::{
    core::{Function, StabilityMap, Transformation, Measurement},
    metrics::{SymmetricDistance, AgnosticMetric},
    domains::{AllDomain, VectorDomain, Array2Domain, ArrayDDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::{Hashable, Primitive},
    trans::{make_row_by_row_fallible, make_row_by_row, postprocess::make_postprocess},
};

fn dataframe_to_array<K: Hashable, TOA: Primitive>(
    col_names: &Vec<K>,
    dataframe: &DataFrame<K>,
) -> Fallible<Array2<TOA>> {

    // collect dataframe into vector of vectors
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

    // convert vector of vectors into array and transpose
    let arr = Array2::from_shape_vec((col_names.len(), nrows), flat)
        .unwrap()
        .reversed_axes();

    return Ok(arr);
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

pub fn make_bin_grid_array2(
    lower_edges: Vec<f64>,
    upper_edges: Vec<f64>,
    bin_count: usize,
) -> Fallible<Transformation<
        Array2Domain<AllDomain<f64>>,
        Array2Domain<AllDomain<usize>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
> {
    use crate::traits::RoundCast;

    let bin_count = bin_count as f64;
    make_row_by_row_fallible(
        AllDomain::new(),
        AllDomain::new(),
        move |row: &Vec<f64>| {
            row.iter()
                .zip(lower_edges.iter().zip(upper_edges.iter()))
                .map(|(v, (l, u))| usize::round_cast(((v - l) / (u - l) * bin_count).floor()))
                .collect()
        },
    )
}

pub fn make_ravel_multi_index(category_lengths: Vec<usize>) -> Fallible<Transformation<
    Array2Domain<AllDomain<usize>>, 
    VectorDomain<AllDomain<usize>>, 
    SymmetricDistance, 
    SymmetricDistance
>> {
    let mut max_index = 1;
    let mut offsets = category_lengths.iter().map(|len| {
        let old_max_index = max_index;
        max_index *= len;
        old_max_index
    }).collect::<Vec<usize>>();

    offsets.reverse();
    let rev_offsets = Array1::from_vec(offsets);

    make_row_by_row(
        AllDomain::new(),
        AllDomain::new(),
        move |row: &Vec<usize>| (&rev_offsets).dot(&Array1::from_vec(row.clone())))
}

pub fn reshape(counts: Vec<usize>, category_lengths: Vec<usize>) -> Fallible<ArrayD<usize>> {
    let reshaped = ArrayD::from_shape_vec(category_lengths, counts);
    return reshaped.map_err(|e| err!(FailedFunction, "{:?}", e));
}

pub fn make_reshape(category_lengths: Vec<usize>) -> Fallible<Transformation<
    VectorDomain<AllDomain<usize>>, 
    ArrayDDomain<AllDomain<usize>>, 
    AgnosticMetric, 
    AgnosticMetric
>> {
    make_postprocess(
        VectorDomain::new_all(),
        ArrayDDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<usize>| reshape(arg.clone(), category_lengths.clone()))
    )
}

pub fn make_repeat_categories<T: Primitive>(categories: Vec<Vec<T>>) -> Fallible<Transformation<
    ArrayDDomain<AllDomain<usize>>, 
    Array2Domain<AllDomain<T>>, 
    AgnosticMetric, 
    AgnosticMetric
>> {

    let cat_shape = (categories.iter())
        .map(Vec::len)
        .collect::<Vec<_>>();
    
    make_postprocess(
        ArrayDDomain::new_all(),
        Array2Domain::new_all(),
        Function::new_fallible(move |counts: &ArrayD<usize>| {
            if counts.shape() != cat_shape {
                return fallible!(FailedFunction, "counts must be the same shape as categories!")
            }

            let data = counts.indexed_iter()
                .flat_map(|(index, &value)| {
                    let row = (categories.iter())
                        .zip(index.as_array_view())
                        .map(|(cats, &i)| cats[i].clone())
                        .collect::<Vec<_>>();
                    vec![row; value.max(0)]
                })
                .flatten()
                .collect::<Vec<T>>();

            Array2::from_shape_vec((counts.sum(), counts.ndim()), data)
                .map_err(|_| err!(FailedFunction, "irregular shape"))
        })
    )
}

use crate::trans::make_count_by_categories;
use crate::meas::make_base_geometric;
use crate::metrics::L1Distance;
use ndarray::Array;

pub fn make_synthetic_discretization<K: Hashable, TOA: Primitive>(
    bin_count: usize,
    lower_edges: Vec<f64>,
    upper_edges: Vec<f64>,
    scale: f64,
) -> Fallible<
    Measurement<
        Array2Domain<AllDomain<f64>>,
        Array2Domain<AllDomain<f64>>,
        SymmetricDistance,
        MaxDivergence<f64>,
    >,
> {
    // build midpoint categories
    let it = upper_edges.iter().zip(lower_edges.iter());
    let categories: Vec<Vec<f64>> = it.map(|(lower, upper)| {
        let offset = (upper - lower) / ((bin_count as f64) * 2.);
        Array::linspace(lower + offset, upper - offset, bin_count).to_vec()
    }).collect();

    let chained = (
        make_bin_grid_array2(lower_edges.clone(), upper_edges.clone(), bin_count)? >>
        make_ravel_multi_index(vec![bin_count, bin_count])? >>
        make_count_by_categories::<L1Distance<usize>, usize, usize>((0..bin_count*bin_count - 1).collect())? >>
        make_base_geometric::<VectorDomain<AllDomain<usize>>, f64>(scale, None)? >>
        make_reshape(vec![bin_count; lower_edges.len()])? >>
        make_repeat_categories(categories)?
    )?;
    return Ok(chained);
}

#[cfg(test)]
mod test {
    use ndarray::{arr2, Ix2};
    use crate::metrics::L1Distance;
    use crate::trans::make_count_by_categories;
    use crate::meas::make_base_geometric;


    use crate::data::Column;
    use super::*;

    #[test]
    fn test_make_synthetic_discretization() -> Fallible<()> {
        let trans = make_synthetic_discretization(3,vec![0., 0., 0.], vec![3., 3., 3.], 1.);
        Ok(())
    }

    #[test]
    fn test_make_select_array() -> Fallible<()> {
        let trans = make_select_array(vec![1, 2, 3])?;

        let mut df = DataFrame::new();
        df.insert(1, Column::new(vec![1., 2., 3.]));
        df.insert(2, Column::new(vec![4., 5., 6.]));
        df.insert(3, Column::new(vec![7., 8., 9.]));

        let arr: Array2<f64> = trans.invoke(&df)?;

        println!("{:?}", arr);

        let expected = arr2(&
            [[1., 4., 7.],
            [2., 5., 8.],
            [3., 6., 9.]]);
        assert_eq!(arr, expected);
        Ok(())
    }

    #[test]
    fn test_make_bin_grid_array2() -> Fallible<()> {
        let trans = make_bin_grid_array2(
            vec![-1., -1.], 
            vec![1., 1.], 
            2)?;
        let array_2 = arr2(&
            [[0.1, 0.1],
            [-0.5, -0.5], 
            [0.5, -0.5]]);
        let arr: Array2<usize> = trans.invoke(&array_2).unwrap();

        println!("{:?}", arr);
        let expected = arr2(&
            [[1, 1],
            [0, 0],
            [1, 0]]);

        assert_eq!(arr, expected);
        Ok(())
    }

    #[test]
    fn test_make_ravel_multi_index() -> Fallible<()> {
        let trans = make_ravel_multi_index(vec![3, 3])?;
        let arr = arr2(&
            [[0, 0],
            [1, 1], 
            [2, 2], 
            [1, 2]]);
        let raveled = trans.invoke(&arr).unwrap();

        println!("{:?}", raveled);
        let expected = vec![0, 4, 8, 5];

        assert_eq!(raveled, expected);
        Ok(())
    }

    #[test]
    fn test_make_reshape() -> Fallible<()> {
        let trans = make_reshape(vec![3, 3])?;
        let vec = vec![1, 1, 0, 1, 0, 0, 0, 1, 2];
        let reshaped = trans.invoke(&vec).unwrap();

        println!("{:?}", reshaped);
        let expected = arr2(&
            [[1, 1, 0],
            [1, 0, 0],
            [0, 1, 2]]);

        assert_eq!(reshaped.into_dimensionality::<Ix2>().unwrap(), expected);
        Ok(())
    }

    #[test]
    fn test_make_repeat_categories() -> Fallible<()> {
        let synth_trans = make_repeat_categories(vec![vec![-0.5f64, 0.5], vec![-0.5, 0.5]])?;
        let reshape_trans = make_reshape(vec![2, 2])?;
        let vec = vec![1, 1, 0, 2];

        let reshaped = reshape_trans.invoke(&vec).unwrap();
        let synthetic = synth_trans.invoke(&reshaped).unwrap();

        println!("{:?}", synthetic);
        let expected = arr2(&
            [[-0.5, -0.5],
            [-0.5, 0.5],
            [0.5, 0.5],
            [0.5, 0.5]]);

        assert_eq!(synthetic, expected);
        Ok(())
    }

    #[test]
    fn test() -> Fallible<()> {
        Ok(())
    }

    #[test]
    fn synthetic_data_pipeline() -> Fallible<()> {

        
        
        let mut df = DataFrame::new();

        // df.insert(1, Column::new(vec![0.1, 1.1, 1.9]));
        // df.insert(2, Column::new(vec![0.1, 1.2, 1.8,]));
        df.insert(1, Column::new(vec![0.1, 1., 1.1]));
        df.insert(2, Column::new(vec![2.9, 0.1, 0.1]));
        
        let bin_count = 3;

        let chained2 = (
            // trans/array
            make_select_array(vec![1i32, 2i32])? >>
            // trans/array
            make_bin_grid_array2(vec![0., 0.], vec![3., 3.], bin_count)? >>
            // trans/array
            make_ravel_multi_index(vec![bin_count, bin_count])? >>

            make_count_by_categories::<L1Distance<usize>, usize, usize>((0..bin_count*bin_count - 1).collect())? >>
            make_base_geometric::<VectorDomain<AllDomain<usize>>, f64>(0.1, Some((0, 10)))? >>
            // // trans/array
            make_reshape(vec![bin_count, bin_count])? >>
            // // trans/array
            make_repeat_categories(vec![vec![0.5, 1.5, 2.5], vec![0.5, 1.5, 2.5]])?
        )?;

        println!("{:?}", chained2.invoke(&df)?);

        // let chained = (df_to_array >> binned_grid)?;
        // println!("{:?}", chained.invoke(&df)?);

        Ok(())
        
    }

}
