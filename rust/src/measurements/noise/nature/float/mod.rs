use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig, ubig,
};

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, LpDistance},
    traits::{CastInternalRational, ExactIntCast, Float, FloatBits, Number},
    transformations::{make_vec, then_index_or_default},
};

#[cfg(test)]
mod test;

pub struct FloatExpFamily<const P: usize> {
    pub scale: f64,
    pub k: i32,
}

/// Float vector mechanism
impl<MO: 'static + Measure, T: Float, const P: usize, QI: Number>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO> for FloatExpFamily<P>
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, LpDistance<P, QI>, MO>> {
        let FloatExpFamily { scale, k } = self;
        let distribution = ZExpFamily {
            scale: integerize_scale(scale, k)?,
        };

        let size = input_domain.size;
        let rounding_distance = get_rounding_distance::<T>(k, size)?;

        let t_int = Transformation::new(
            input_domain,
            VectorDomain {
                element_domain: AtomDomain::<IBig>::default(),
                size,
            },
            Function::new(move |arg: &Vec<T>| {
                arg.into_iter()
                    .cloned()
                    .map(|x_i| {
                        let x_i = RBig::try_from(x_i).unwrap_or(RBig::ZERO);
                        find_nearest_multiple_of_2k(x_i, k)
                    })
                    .collect()
            }),
            input_metric.clone(),
            LpDistance::default(),
            StabilityMap::new_fallible(move |d_in: &QI| {
                let d_in = RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({:?}) must be finite", d_in))?;
                Ok(x_div_2k(d_in + rounding_distance.clone(), k))
            }),
        )?;

        let m_noise = distribution.make_noise(t_int.output_space())?;

        t_int >> m_noise >> then_deintegerize_vec(self.k)
    }
}

/// Float scalar mechanism
impl<MO: 'static + Measure, T: Float, const P: usize, QI: Number>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO> for FloatExpFamily<P>
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (AtomDomain<T>, AbsoluteDistance<QI>),
    ) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<QI>, MO>> {
        let t_vec = make_vec(input_space)?;
        let m_noise = self.make_noise(t_vec.output_space())?;

        t_vec >> m_noise >> then_index_or_default(0)
    }
}

pub fn then_deintegerize_vec<TO: CastInternalRational>(k: i32) -> Function<Vec<IBig>, Vec<TO>> {
    Function::new(move |x: &Vec<IBig>| {
        x.into_iter()
            .cloned()
            .map(|x_i| TO::from_rational(x_mul_2k(x_i, k)))
            .collect()
    })
}

pub(crate) fn get_min_k<T: Float>() -> i32
where
    i32: ExactIntCast<T::Bits>,
{
    -i32::exact_int_cast(T::EXPONENT_BIAS).unwrap() - i32::exact_int_cast(T::MANTISSA_BITS).unwrap()
        + 1
}

pub fn get_rounding_distance<T: Float>(k: i32, size: Option<usize>) -> Fallible<RBig>
where
    i32: ExactIntCast<T::Bits>,
{
    let k_min = get_min_k::<T>();
    if k < k_min {
        return fallible!(FailedFunction, "k ({k}) must not be smaller than {k_min}");
    }

    // input has granularity 2^{k_min} (subnormal float precision)
    let input_gran = x_div_2k(rbig!(2), -k_min);

    // discretization rounds to the nearest 2^k
    let output_gran = x_div_2k(rbig!(2), -k);

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
pub fn find_nearest_multiple_of_2k(x: RBig, k: i32) -> IBig {
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
pub fn x_div_2k(x: RBig, k: i32) -> RBig {
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
