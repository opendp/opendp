use dashu::{integer::IBig, rational::RBig};
use opendp_derive::proven;

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, LpDistance},
    traits::{Integer, Number, SaturatingCast},
    transformations::{make_vec, then_index_or_default},
};

use super::float::integerize_scale;

#[cfg(test)]
mod test;

pub struct IntExpFamily<const P: usize> {
    pub scale: f64,
}

/// # Proof Definition
/// For any choice of arguments, returns a valid measurement.
#[proven(proof_path = "measurements/noise/nature/integer/make_int_to_bigint.tex")]
fn make_int_to_bigint<T: Integer, const P: usize, QI: Number>(
    (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        LpDistance<P, QI>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<P, RBig>,
    >,
>
where
    IBig: From<T>,
    RBig: TryFrom<QI>,
{
    Transformation::new(
        input_domain.clone(),
        input_metric,
        VectorDomain {
            element_domain: AtomDomain::<IBig>::default(),
            size: input_domain.size.clone(),
        },
        LpDistance::default(),
        Function::new(move |x: &Vec<T>| x.iter().cloned().map(IBig::from).collect()),
        StabilityMap::new_fallible(move |&d_in: &QI| {
            RBig::try_from(d_in).map_err(|_| err!(FailedMap, "d_in ({d_in:?}) must be finite"))
        }),
    )
}

/// Integer scalar mechanism
#[proven(
    proof_path = "measurements/noise/nature/integer/MakeNoise_AtomDomain_for_IntExpFamily.tex"
)]
impl<T, const P: usize, QI, MO> MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
    for IntExpFamily<P>
where
    T: Integer + SaturatingCast<IBig>,
    QI: Number,
    MO: 'static + Measure,
    IBig: From<T>,
    RBig: TryFrom<QI>,
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

/// Integer vector mechanism
#[proven(
    proof_path = "measurements/noise/nature/integer/MakeNoise_VectorDomain_for_IntExpFamily.tex"
)]
impl<T, const P: usize, QI: Number, MO>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO> for IntExpFamily<P>
where
    T: Integer + SaturatingCast<IBig>,
    MO: 'static + Measure,
    IBig: From<T>,
    RBig: TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO, Vec<T>>> {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
        };

        let t_int = make_int_to_bigint(input_space)?;
        let m_noise = distribution.make_noise(t_int.output_space())?;
        t_int >> m_noise >> then_saturating_cast()
    }
}

/// # Proof Definition
/// For any choice of arguments, returns a valid postprocessor.
#[proven(proof_path = "measurements/noise/nature/integer/then_saturating_cast.tex")]
fn then_saturating_cast<TO: SaturatingCast<IBig>>() -> Function<Vec<IBig>, Vec<TO>> {
    Function::new(move |x: &Vec<IBig>| x.into_iter().cloned().map(TO::saturating_cast).collect())
}
