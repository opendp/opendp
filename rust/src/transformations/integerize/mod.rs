use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig, ubig,
};

use crate::{
    error::Fallible,
    traits::{ExactIntCast, Float},
};

mod vec;
pub use vec::*;

mod hashmap;
pub use hashmap::*;

pub(crate) fn get_min_k<T: Float>() -> i32
where
    i32: ExactIntCast<T::Bits>,
{
    -i32::exact_int_cast(T::EXPONENT_BIAS).unwrap() - i32::exact_int_cast(T::MANTISSA_BITS).unwrap()
        + 1
}

fn get_rounding_distance<T: Float>(k: i32, size: Option<usize>) -> Fallible<RBig>
where
    i32: ExactIntCast<T::Bits>,
{
    let k_min = get_min_k::<T>();
    if k < k_min {
        return fallible!(FailedFunction, "k ({k}) must not be smaller than {k_min}");
    }

    // input has granularity 2^{k_min} (subnormal float precision)
    let input_gran = rbig!(2).pow(k_min as usize);

    // discretization rounds to the nearest 2^k
    let output_gran = rbig!(2).pow(k as usize);

    // the worst-case increase in sensitivity due to discretization is
    //     the range, minus the smallest step in the range
    let mut distance = output_gran - input_gran;

    // rounding may occur on all vector elements
    if !distance.is_zero() {
        let size = size.ok_or_else(|| {
            err!(
                MakeMeasurement,
                "domain size must be known if discretization is not exact"
            )
        })?;
        distance *= RBig::from(size);
    }
    Ok(distance)
}

pub fn integerize_scale(scale: f64, k: i32) -> Fallible<RBig> {
    let scale = RBig::try_from(scale)
        .map_err(|_| err!(MakeTransformation, "scale ({scale}) must be finite"))?;

    Ok(x_div_2k(scale, k))
}

/// Find index of nearest multiple of $2^k$ from x.
///
/// # Proof Definition
/// For any setting of input arguments, return the integer $argmin_i |i 2^k - x|$.
fn find_nearest_multiple_of_2k(x: RBig, k: i32) -> IBig {
    // exactly compute x/2^k and break into fractional parts
    let (numer, denom) = x_div_2k(x, k).into_parts();

    // argmin_i |i * 2^k - x|, the index of nearest multiple of 2^k
    let offset = &denom / ubig!(2) * numer.sign();
    (numer + offset) / denom
}

/// Find index of nearest multiple of $2^k$ from x.
///
/// # Proof Definition
/// For any setting of input arguments, return the integer $argmin_i |i 2^k - x|$.
pub(crate) fn find_next_multiple_of_2k(x: RBig, k: i32) -> IBig {
    // exactly compute x/2^k and break into fractional parts
    let (numer, denom) = x_div_2k(x, k).into_parts();

    let offset = denom.clone() * numer.sign();
    (numer + offset) / denom
}

/// # Proof Definition
/// Divide `x` by 2^`k` exactly.
fn x_div_2k(x: RBig, k: i32) -> RBig {
    let (mut num, mut den) = x.into_parts();
    if k < 0 {
        num <<= -k as usize;
    } else {
        den <<= k as usize;
    }

    RBig::from_parts(num, den)
}

/// Exactly multiply x by 2^k.
///
/// This is a postprocessing operation.
pub(crate) fn x_mul_2k(x: IBig, k: i32) -> RBig {
    if k > 0 {
        RBig::from(x << k as usize)
    } else {
        RBig::from_parts(x, UBig::ONE << -k as usize)
    }
}
