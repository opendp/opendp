use crate::{
    core::Measurement,
    metrics::{SymmetricDistance, L1Distance},
    domains::{AllDomain, VectorDomain, Array2Domain},
    error::Fallible,
    measurements::make_base_geometric,
    measures::MaxDivergence,
    transformations::make_count_by_categories,
    transformations::array::{make_bin_grid_array2, make_ravel_multi_index, make_reshape, make_repeat_categories},
};

use ndarray::Array;

pub fn make_synthetic_discretization(
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
    let it = lower_edges.iter().zip(upper_edges.iter());
    let categories: Vec<Vec<f64>> = it.map(|(lower, upper)| {
        let offset = (upper - lower) / ((bin_count as f64) * 2.);
        Array::linspace(lower + offset, upper - offset, bin_count).to_vec()
    }).collect();

    // have to check that dim == arr.ncols()
    let dim: u32 = lower_edges.len() as u32;

    let chained = (
        make_bin_grid_array2(lower_edges.clone(), upper_edges.clone(), bin_count)? >>
        make_ravel_multi_index(vec![bin_count; lower_edges.len()])? >>
        make_count_by_categories::<L1Distance<usize>, usize, usize>((0..bin_count.pow(dim) - 1).collect())? >>
        make_base_geometric::<VectorDomain<AllDomain<usize>>, f64>(scale, None)? >>
        make_reshape(vec![bin_count; lower_edges.len()])? >>
        make_repeat_categories(categories)?
    )?;
    return Ok(chained);
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_make_synthetic_discretization() -> Fallible<()> {
        
        let arr = arr2(&
            [[2.8, 2.7, 2.5],     
            [1.3, 1.7, 1.8],
            [0.5, 1.5, 2.5],
            [0.1, 0.3, 0.4]],);

        let trans = make_synthetic_discretization(
            3, 
            vec![0., 0., 0.], 
            vec![3., 3., 3.], 
            0.1)?;

        let synthetic = trans.invoke(&arr).unwrap();

        println!("{:?}", synthetic);
        let expected = arr2(&
            [[0.5, 0.5, 0.5],
            [0.5, 1.5, 2.5],
            [1.5, 1.5, 1.5],
            [2.5, 2.5, 2.5]],);
        assert_eq!(synthetic, expected);

        Ok(())
        
    }
}