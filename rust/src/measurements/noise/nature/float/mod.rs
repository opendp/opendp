use dashu::{integer::IBig, rational::RBig};
use opendp_derive::proven;

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, LpDistance},
    traits::{CastInternalRational, ExactIntCast, Float, FloatBits, Number},
    transformations::{make_vec, then_index_or_default},
};

mod utilities;
pub(crate) use utilities::*;

#[cfg(test)]
mod test;

pub struct FloatExpFamily<const P: usize> {
    pub scale: f64,
    pub k: i32,
}

/// Float vector mechanism
#[proven(
    proof_path = "measurements/noise/nature/float/MakeNoise_VectorDomain_for_FloatExpFamily.tex"
)]
impl<T: Float, const P: usize, QI: Number, MO: 'static + Measure>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO> for FloatExpFamily<P>
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO, Vec<T>>> {
        let FloatExpFamily { scale, k } = self;
        let distribution = ZExpFamily {
            scale: integerize_scale(scale, k)?,
        };

        let t_int = make_float_to_bigint(input_space, k)?;
        let m_noise = distribution.make_noise(t_int.output_space())?;
        t_int >> m_noise >> then_deintegerize_vec(self.k)?
    }
}

#[proven(proof_path = "measurements/noise/nature/float/make_float_to_bigint.tex")]
fn make_float_to_bigint<T: Float, const P: usize, QI: Number>(
    input_space: (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    k: i32,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        LpDistance<P, QI>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<P, RBig>,
    >,
>
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;
    if input_domain.element_domain.nan() {
        return fallible!(
            MakeTransformation,
            "input_domain may not contain NaN elements"
        );
    }
    let size = input_domain.size;
    let rounding_distance = get_rounding_distance::<T, P>(k, size)?;

    Transformation::new(
        input_domain,
        input_metric.clone(),
        VectorDomain {
            element_domain: AtomDomain::<IBig>::default(),
            size,
        },
        LpDistance::default(),
        Function::new(move |arg: &Vec<T>| {
            arg.iter()
                .cloned()
                .map(|x_i| {
                    let x_i = RBig::try_from(x_i.clamp(T::MIN_FINITE, T::MAX_FINITE))
                        .unwrap_or(RBig::ZERO);
                    find_nearest_multiple_of_2k(x_i, k)
                })
                .collect()
        }),
        StabilityMap::new_fallible(move |d_in: &QI| {
            let d_in = RBig::try_from(d_in.clone())
                .map_err(|_| err!(FailedMap, "d_in ({:?}) must be finite", d_in))?;
            Ok(x_mul_2k(d_in + rounding_distance.clone(), -k))
        }),
    )
}

/// Float scalar mechanism
#[proven(
    proof_path = "measurements/noise/nature/float/MakeNoise_AtomDomain_for_FloatExpFamily.tex"
)]
impl<T: Float, const P: usize, QI: Number, MO: 'static + Measure>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO> for FloatExpFamily<P>
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (AtomDomain<T>, AbsoluteDistance<QI>),
    ) -> Fallible<Measurement<AtomDomain<T>, AbsoluteDistance<QI>, MO, T>> {
        let t_vec = make_vec(input_space)?;
        let m_noise = self.make_noise(t_vec.output_space())?;

        t_vec >> m_noise >> then_index_or_default(0)
    }
}

#[proven(proof_path = "measurements/noise/nature/float/then_deintegerize_vec.tex")]
pub fn then_deintegerize_vec<TO: CastInternalRational>(
    k: i32,
) -> Fallible<Function<Vec<IBig>, Vec<TO>>> {
    if k == i32::MIN {
        return fallible!(MakeTransformation, "k must not be i32::MIN");
    }
    Ok(Function::new(move |x: &Vec<IBig>| {
        x.iter()
            .cloned()
            .map(|x_i| TO::from_rational(x_mul_2k(RBig::from(x_i), k)))
            .collect()
    }))
}
