#[cfg(feature = "ffi")]
mod ffi;

use num::{Float as _, Zero};
use opendp_derive::bootstrap;

use crate::core::Transformation;
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, SymmetricDistance};
use crate::traits::{AlertingSub, ExactIntCast, Float, InfDiv, InfMul, InfPowI, InfSub};

use super::{
    make_lipschitz_float_mul, make_sum_of_squared_deviations, LipschitzMulFloatDomain,
    LipschitzMulFloatMetric, Pairwise, UncheckedSum,
};

#[bootstrap(
    features("contrib"),
    arguments(bounds(rust_type = "(T, T)"), ddof(default = 1)),
    generics(S(default = "Pairwise<T>", generics = "T")),
    derived_types(T = "$get_atom(get_type(input_domain))")
)]
/// Make a Transformation that computes the variance of bounded data.
///
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size.
/// Use `make_clamp` to bound data and `make_resize` to establish dataset size.
///
/// # Citations
/// * [DHK15 Differential Privacy for Social Science Inference](http://hona.kr/papers/files/DOrazioHonakerKingPrivacy.pdf)
///
/// # Arguments
/// * `size` - Number of records in input data.
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
/// * `ddof` - Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
///
/// # Generics
/// * `S` - Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
pub fn make_variance<S>(
    input_domain: VectorDomain<AtomDomain<S::Item>>,
    input_metric: SymmetricDistance,
    ddof: usize,
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
    AtomDomain<S::Item>: LipschitzMulFloatDomain<Atom = S::Item>,
    AbsoluteDistance<S::Item>: LipschitzMulFloatMetric<Distance = S::Item>,
{
    let size = (input_domain.size)
        .ok_or_else(|| err!(MakeTransformation, "dataset size must be known. Either specify size in the input domain or use make_resize"))?;
    let bounds = (input_domain.element_domain.get_closed_bounds())?;
    if ddof >= size {
        return fallible!(MakeTransformation, "size - ddof must be greater than zero");
    }

    let constant = S::Item::exact_int_cast(size.alerting_sub(&ddof)?)?.recip();
    let _4 = S::Item::exact_int_cast(4)?;
    let size_ = S::Item::exact_int_cast(size)?;

    // Using Popoviciu's inequality on variances:
    //     variance <= (U - L)^2 / 4
    // Therefore ssd <= variance * size <= (U - L)^2 / 4 * size
    let upper_var_bound = (bounds.1)
        .inf_sub(&bounds.0)?
        .inf_powi(2.into())?
        .inf_div(&_4)?
        .inf_mul(&size_)?;

    make_sum_of_squared_deviations::<Pairwise<_>>(input_domain, input_metric)?
        >> make_lipschitz_float_mul(constant, (S::Item::zero(), upper_var_bound))?
}

#[cfg(test)]
mod tests {
    use crate::transformations::Pairwise;

    use super::*;

    #[test]
    fn test_make_variance() -> Fallible<()> {
        let arg = vec![1., 2., 3., 4., 5.];

        let input_domain = VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(5);
        let input_metric = SymmetricDistance::default();
        let transformation_sample =
            make_variance::<Pairwise<_>>(input_domain.clone(), input_metric.clone(), 1)?;
        let ret = transformation_sample.invoke(&arg)?;
        let expected = 2.5;
        assert_eq!(ret, expected);
        assert!(transformation_sample.check(&1, &(100. / 5.))?);

        let transformation_pop = make_variance::<Pairwise<_>>(input_domain, input_metric, 0)?;
        let ret = transformation_pop.invoke(&arg)?;
        let expected = 2.0;
        assert_eq!(ret, expected);
        assert!(transformation_pop.check(&1, &(100. * 4. / 25.))?);

        Ok(())
    }
}
