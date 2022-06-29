use crate::{
    core::{StabilityMap, Transformation, Function},
    dist::{AbsoluteDistance, SymmetricDistance},
    dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::{ExactIntCast, InfCast, InfAdd, InfDiv, InfMul, InfSub},
};

use num::{Zero, One};

use super::{Float, UncheckedSum};


#[cfg(feature="ffi")]
pub mod ffi;


type CovarianceDomain<T> = SizedDomain<VectorDomain<BoundedDomain<(T, T)>>>;

pub fn make_sized_bounded_covariance<S>(
    size: usize,
    bounds_0: (S::Item, S::Item),
    bounds_1: (S::Item, S::Item),
    ddof: usize,
) -> Fallible<
    Transformation<
        CovarianceDomain<S::Item>,
        AllDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float
{
    if size == 0 {
        return fallible!(MakeTransformation, "size must be greater than zero")
    }
    if ddof >= size {
        return fallible!(MakeTransformation, "size - ddof must be greater than zero")
    }
    let _size = S::Item::exact_int_cast(size)?;
    let _ddof = S::Item::exact_int_cast(ddof)?;
    let (lower_0, upper_0) = bounds_0.clone();
    let (lower_1, upper_1) = bounds_1.clone();
    let _1 = S::Item::one();
    let _2 = S::Item::exact_int_cast(2)?;

    // DERIVE RELAXATION TERM
    // Let x_bar_approx = x_bar + 2e, the approximate mean on finite data types
    // Let e = (n^2/2^k) / n, the mean error
    let mean_0_error = S::error(size, lower_0, upper_0)?.inf_div(&_size)?;
    let mean_1_error = S::error(size, lower_1, upper_1)?.inf_div(&_size)?;

    // Let L' = L - e, U' = U + e
    let (lower_0, upper_0) = (
        lower_0.neg_inf_sub(&mean_0_error)?,
        upper_0.inf_add(&mean_0_error)?,
    );
    let (lower_1, upper_1) = (
        lower_1.neg_inf_sub(&mean_1_error)?,
        upper_1.inf_add(&mean_1_error)?,
    );

    // Let range = U' - L'
    let range_0 = upper_0.inf_sub(&lower_0)?;
    let range_1 = upper_1.inf_sub(&lower_1)?;

    // Let sens = range_0 * range_1 * (n - 1) / n
    let sensitivity = range_0
        .inf_mul(&range_1)?
        .inf_mul(&_size.inf_sub(&_1)?)?
        .inf_div(&_size)?
        .inf_div(&_size.neg_inf_sub(&_ddof)?)?;

    let relaxation = S::relaxation(size, S::Item::zero(), range_0.inf_mul(&range_1)?)?;

    // OVERFLOW CHECKS
    // Bound the magnitudes of the sums when computing the means
    bounds_0.0.inf_mul(&_size)?;
    bounds_0.1.inf_mul(&_size)?;
    bounds_1.0.inf_mul(&_size)?;
    bounds_1.1.inf_mul(&_size)?;
    // The squared difference from the mean is bounded above by range^2
    range_0.inf_mul(&range_1)?.inf_mul(&_size)?;

    Ok(Transformation::new(
        SizedDomain::new(
            VectorDomain::new(BoundedDomain::new_closed((
                (bounds_0.0.clone(), bounds_1.0.clone()),
                (bounds_0.1.clone(), bounds_1.1.clone()),
            ))?),
            size,
        ),
        AllDomain::new(),
        Function::new(enclose!(_size, move |arg: &Vec<(S::Item, S::Item)>| {
            let (l, r): (Vec<S::Item>, Vec<S::Item>) = arg.iter().copied().unzip();
            let (sum_l, sum_r) = (S::unchecked_sum(&l), S::unchecked_sum(&r));
            let (mean_l, mean_r) = (sum_l / _size, sum_r / _size);

            let ssd = S::unchecked_sum(&arg.iter().copied()
                .map(|(v_l, v_r)| (v_l - mean_l) * (v_r - mean_r)).collect::<Vec<S::Item>>());
            
            ssd / (_size - _ddof)
        })),
        SymmetricDistance::default(),
        AbsoluteDistance::default(),
        // d_in / 2 * sensitivity + relaxation
        StabilityMap::new_fallible(move |d_in| {
            S::Item::inf_cast(d_in / 2)?
                .inf_mul(&sensitivity)?
                .inf_add(&relaxation)
        }),
    ))
}

#[cfg(test)]
mod test {
    use crate::trans::Pairwise;

    use super::*;

    #[test]
    fn test_make_bounded_covariance_hamming() -> Fallible<()> {
        let arg = vec![(1., 3.), (2., 4.), (3., 5.), (4., 6.), (5., 7.)];

        let transformation_sample =
            make_sized_bounded_covariance::<Pairwise<f64>>(5, (0., 2.), (10., 12.), 1)?;
        let ret = transformation_sample.invoke(&arg)?;
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.))?);

        let transformation_pop =
            make_sized_bounded_covariance::<Pairwise<f64>>(5, (0., 2.), (10., 12.), 0)?;
        let ret = transformation_pop.invoke(&arg)?;
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop
            .check(&1, &(100. * 4. / 25.))?);
        Ok(())
    }
}
