use opendp_derive::bootstrap;

use crate::{
    core::{Function, StabilityMap, Transformation},
    metrics::{AbsoluteDistance, InsertDeleteDistance, IntDistance},
    domains::{AllDomain, BoundedDomain, SizedDomain, VectorDomain},
    error::Fallible,
    traits::Number,
};

use super::AddIsExact;

#[cfg(feature = "ffi")]
mod ffi;


#[bootstrap(
    features("contrib"),
    generics(T(example = "$get_first(bounds)"))
)]
/// Make a Transformation that computes the sum of bounded ints.
/// You may need to use `make_ordered_random` to impose an ordering on the data.
/// 
/// # Citations
/// * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
/// * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
/// 
/// # Arguments
/// * `bounds` - Tuple of lower and upper bounds for data in the input domain.
/// 
/// # Generics
/// * `T` - Atomic Input Type and Output Type
pub fn make_bounded_int_ordered_sum<T>(
    bounds: (T, T),
) -> Fallible<
    Transformation<
        VectorDomain<BoundedDomain<T>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: Number + AddIsExact,
{
    let (lower, upper) = bounds.clone();
    Ok(Transformation::new(
        VectorDomain::new(BoundedDomain::new_closed(bounds)?),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(lower.alerting_abs()?.total_max(upper)?),
    ))
}

#[bootstrap(
    features("contrib"),
    generics(T(example = "$get_first(bounds)"))
)]
/// Make a Transformation that computes the sum of bounded ints with known dataset size. 
/// 
/// This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility. 
/// You may need to use `make_ordered_random` to impose an ordering on the data.
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
/// * `T` - Atomic Input Type and Output Type
pub fn make_sized_bounded_int_ordered_sum<T>(
    size: usize,
    bounds: (T, T),
) -> Fallible<
    Transformation<
        SizedDomain<VectorDomain<BoundedDomain<T>>>,
        AllDomain<T>,
        InsertDeleteDistance,
        AbsoluteDistance<T>,
    >,
>
where
    T: Number + AddIsExact,
{
    let (lower, upper) = bounds.clone();
    let range = upper.inf_sub(&lower)?;
    Ok(Transformation::new(
        SizedDomain::new(VectorDomain::new(BoundedDomain::new_closed(bounds)?), size),
        AllDomain::new(),
        Function::new(|arg: &Vec<T>| arg.iter().fold(T::zero(), |sum, v| sum.saturating_add(v))),
        InsertDeleteDistance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(
            // If d_in is odd, we still only consider databases with (d_in - 1) / 2 substitutions,
            //    so floor division is acceptable
            move |d_in: &IntDistance| T::inf_cast(d_in / 2).and_then(|d_in| d_in.inf_mul(&range)),
        ),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_make_bounded_int_ordered_sum() -> Fallible<()> {
        let trans = make_bounded_int_ordered_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_bounded_int_ordered_sum((1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        // test saturation arithmetic
        let trans = make_bounded_int_ordered_sum((1i8, 127))?;
        let sum = trans.invoke(&vec![-128, -128, 127, 127, 127])?;
        assert_eq!(sum, 127);

        Ok(())
    }

    #[test]
    fn test_make_sized_bounded_int_ordered_sum() -> Fallible<()> {
        let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        let trans = make_sized_bounded_int_ordered_sum(4, (1i32, 10))?;
        let sum = trans.invoke(&vec![1, 2, 3, 4])?;
        assert_eq!(sum, 10);

        Ok(())
    }
}
