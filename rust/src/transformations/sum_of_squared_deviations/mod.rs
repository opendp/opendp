#[cfg(feature = "ffi")]
mod ffi;

use num::{Float as _, One, Zero};
use opendp_derive::bootstrap;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::{ExactIntCast, Float, InfAdd, InfCast, InfDiv, InfMul, InfSub};

use super::UncheckedSum;

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)")),
    generics(S(default = "Pairwise<T>", generics = "T")),
    derived_types(T = "$get_atom(get_type(input_domain))")
)]
/// Make a Transformation that computes the sum of squared deviations of bounded data.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size.
/// Use `make_clamp` to bound data and `make_resize` to establish dataset size.
///
/// | S (summation algorithm) | input type     |
/// | ----------------------- | -------------- |
/// | `Sequential<S::Item>`   | `Vec<S::Item>` |
/// | `Pairwise<S::Item>`     | `Vec<S::Item>` |
///
/// `S::Item` is the type of all of the following:
/// each bound, each element in the input data, the output data, and the output sensitivity.
///
/// For example, to construct a transformation that computes the SSD of `f32` half-precision floats,
/// set `S` to `Pairwise<f32>`.
///
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
///
/// # Generics
/// * `S` - Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
pub fn make_sum_of_squared_deviations<S>(
    input_domain: VectorDomain<AtomDomain<S::Item>>,
    input_metric: SymmetricDistance,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<S::Item>>,
        AtomDomain<S::Item>,
        SymmetricDistance,
        AbsoluteDistance<S::Item>,
    >,
>
where
    S: UncheckedSum,
    S::Item: 'static + Float,
{
    let size = (input_domain.size)
        .ok_or_else(|| err!(MakeTransformation, "dataset size must be known. Either specify size in the input domain or use make_resize"))?;
    let bounds = (input_domain.element_domain.get_closed_bounds())
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "input domain must consist of bounded data. Either specify bounds in the input domain or use make_clamp."
            )
        })?;

    if size == 0 {
        return fallible!(MakeTransformation, "size must be greater than zero");
    }
    let size_ = S::Item::exact_int_cast(size)?;
    let (lower, upper) = bounds;
    let _1 = S::Item::one();

    // DERIVE RELAXATION TERM
    // Let x_bar_approx = x_bar + 2e, the approximate mean on finite data types
    // Let e = (n^2/2^k) / n, the mean error
    let mean_error = S::error(size, lower, upper)?.inf_div(&size_)?;

    // Let L' = L - e, U' = U + e
    let (lower, upper) = (lower.neg_inf_sub(&mean_error)?, upper.inf_add(&mean_error)?);

    // Let range = U' - L'
    let range = upper.inf_sub(&lower)?;

    // Let sens = range^2 * (n - 1) / n
    let sensitivity = range
        .inf_mul(&range)?
        .inf_mul(&size_.inf_sub(&_1)?)?
        .inf_div(&size_)?;

    // each deviation is bounded between 0 and range^2
    let relaxation = S::relaxation(size, S::Item::zero(), range.inf_mul(&range)?)?;

    // OVERFLOW CHECKS
    // Bound the magnitude of the sum when computing the mean
    lower.inf_mul(&size_)?;
    upper.inf_mul(&size_)?;
    // The squared difference from the mean is bounded above by range^2
    range.inf_mul(&range)?.inf_mul(&size_)?;

    Transformation::new(
        input_domain,
        AtomDomain::default(),
        Function::new(move |arg: &Vec<S::Item>| {
            let mean = S::unchecked_sum(arg) / size_;
            S::unchecked_sum(
                &arg.iter()
                    .map(|v| (*v - mean).powi(2))
                    .collect::<Vec<S::Item>>(),
            )
        }),
        input_metric,
        AbsoluteDistance::default(),
        // d_in / 2 * sensitivity + relaxation
        StabilityMap::new_fallible(move |d_in| {
            S::Item::inf_cast(d_in / 2)?
                .inf_mul(&sensitivity)?
                .inf_add(&relaxation)
        }),
    )
}

#[cfg(test)]
mod tests {
    use crate::transformations::Pairwise;

    use super::*;

    #[test]
    fn test_make_bounded_deviations() -> Fallible<()> {
        let arg = vec![1., 2., 3., 4., 5.];

        let input_domain = VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(5);
        let input_metric = SymmetricDistance::default();
        let transformation_sample = make_sum_of_squared_deviations::<Pairwise<_>>(
            input_domain.clone(),
            input_metric.clone(),
        )?;
        let ret = transformation_sample.invoke(&arg)?;
        let expected = 10.;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.))?);

        let transformation_pop =
            make_sum_of_squared_deviations::<Pairwise<_>>(input_domain, input_metric)?;
        let ret = transformation_pop.invoke(&arg)?;
        let expected = 10.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.check(&1, &(100. * 4. / 25.))?);

        Ok(())
    }
}
