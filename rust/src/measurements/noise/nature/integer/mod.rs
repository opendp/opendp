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

pub struct IntExpFamily<const P: usize> {
    pub scale: f64,
}

/// Integer vector mechanism
#[proven(
    proof_path = "measurements/noise/nature/integer/MakeNoise_VectorDomain_for_IntExpFamily.tex"
)]
impl<MO: 'static + Measure, T, const P: usize, QI: Number>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, MO> for IntExpFamily<P>
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    RBig: TryFrom<QI>,
    ZExpFamily<P>: NoisePrivacyMap<LpDistance<P, RBig>, MO>,
{
    fn make_noise(
        self,
        input_space: (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, LpDistance<P, QI>, MO>> {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
        };

        let t_int = make_int_to_bigint(input_space)?;
        let m_noise = distribution.make_noise(t_int.output_space())?;
        t_int >> m_noise >> then_saturating_cast()
    }
}

#[proven(proof_path = "measurements/noise/nature/integer/make_int_to_bigint.tex")]
fn make_int_to_bigint<T: Integer, const P: usize, QI: Number>(
    input_space: (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<P, QI>,
        LpDistance<P, RBig>,
    >,
>
where
    IBig: From<T>,
    RBig: TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;
    Transformation::new(
        input_domain.clone(),
        VectorDomain {
            element_domain: AtomDomain::<IBig>::default(),
            size: input_domain.size.clone(),
        },
        Function::new(move |x: &Vec<T>| x.iter().cloned().map(IBig::from).collect()),
        input_metric,
        LpDistance::default(),
        StabilityMap::new_fallible(move |&d_in: &QI| {
            RBig::try_from(d_in).map_err(|_| err!(FailedMap, "d_in ({d_in:?}) must be finite"))
        }),
    )
}

#[proven(proof_path = "measurements/noise/nature/integer/then_saturating_cast.tex")]
fn then_saturating_cast<T: SaturatingCast<IBig>>() -> Function<Vec<IBig>, Vec<T>> {
    Function::new(move |x: &Vec<IBig>| x.into_iter().cloned().map(T::saturating_cast).collect())
}

/// Integer scalar mechanism
#[proven(
    proof_path = "measurements/noise/nature/integer/MakeNoise_AtomDomain_for_IntExpFamily.tex"
)]
impl<MO, T, const P: usize, QI> MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
    for IntExpFamily<P>
where
    MO: 'static + Measure,
    T: Integer + SaturatingCast<IBig>,
    QI: Number,
    IBig: From<T>,
    RBig: TryFrom<QI>,
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
