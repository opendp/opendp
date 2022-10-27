use std::convert::TryFrom;

use crate::{
    core::Measurement,
    domains::{Array2Domain, RowDomain, SizedDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::SymmetricDistance,
    traits::{ExactIntCast, Float, RoundCast},
    transformations::array::{
        make_bin_grid_array2, make_ravel_multi_index, make_repeat_categories, make_reshape,
    },
    transformations::make_count_by_categories,
    measurements::make_base_discrete_laplace
};

use ndarray::Array;

pub fn make_synthetic_discretization<T>(
    lower_edges: Vec<T>,
    upper_edges: Vec<T>,
    bin_count: usize,
    scale: T,
) -> Fallible<
    Measurement<
        Array2Domain<SizedDomain<RowDomain<T>>>,
        Array2Domain<SizedDomain<RowDomain<T>>>,
        SymmetricDistance,
        MaxDivergence<T>,
    >,
>
where
    T: Float,
    T::Bits: PartialOrd + ExactIntCast<usize>,
    usize: ExactIntCast<T::Bits> + RoundCast<T>,
    rug::Rational: TryFrom<T>
{
    let _2 = T::one() + T::one();
    let bin_count_ = T::exact_int_cast(bin_count)?;

    let shape = vec![bin_count; lower_edges.len()];
    let dim: u32 = lower_edges.len() as u32;
    let usize_cats = (0..bin_count.pow(dim)).collect();

    // build midpoint categories
    let categories: Vec<Vec<T>> = (lower_edges.iter())
        .zip(upper_edges.iter())
        .map(|(&lower, &upper)| {
            let offset = (upper - lower) / ((bin_count_) * _2);
            Array::linspace(lower + offset, upper - offset, bin_count).to_vec()
        })
        .collect();

    make_bin_grid_array2(lower_edges, upper_edges, bin_count)?
        >> make_ravel_multi_index(shape.clone())?
        >> make_count_by_categories(usize_cats, false)?
        >> make_base_discrete_laplace(scale)?
        >> make_reshape(shape)?
        >> make_repeat_categories(categories)?
}

#[cfg(test)]
mod test {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_make_synthetic_discretization() -> Fallible<()> {
        let arr = arr2(&[
            [2.8, 2.7, 2.5],
            [1.3, 1.7, 1.8],
            [0.5, 1.5, 2.5],
            [0.1, 0.3, 0.4],
        ]);

        let trans = make_synthetic_discretization(vec![0., 0., 0.], vec![3., 3., 3.], 3, 0.0)?;

        let synthetic = trans.invoke(&arr).unwrap();

        println!("{:?}", synthetic);
        let expected = arr2(&[
            [0.5, 0.5, 0.5],
            [0.5, 1.5, 2.5],
            [1.5, 1.5, 1.5],
            [2.5, 2.5, 2.5],
        ]);
        assert_eq!(synthetic, expected);

        Ok(())
    }
}
