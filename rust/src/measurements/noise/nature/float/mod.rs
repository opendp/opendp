use dashu::rational::RBig;
use opendp_derive::proven;

use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, LpDistance},
    traits::{
        Float, Number,
        samplers::{IidContinuousGaussian, IidContinuousLaplace, RoundedContinuousVectorSampler},
    },
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
impl<T: Float, QI: Number, MO: 'static + Measure>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<1, QI>, MO> for FloatExpFamily<1>
where
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<1>: NoisePrivacyMap<LpDistance<1, RBig>, MO>,
{
    fn make_noise(
        self,
        (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<1, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LpDistance<1, QI>, MO, Vec<T>>> {
        if input_domain.element_domain.nan() {
            return fallible!(MakeMeasurement, "input_domain may not contain NaN elements");
        }

        let scale = RBig::try_from(self.scale)?;
        let distribution = ZExpFamily {
            scale: scale.clone(),
        };
        let output_measure = MO::default();
        let privacy_map =
            distribution.noise_privacy_map(&LpDistance::<1, RBig>::default(), &output_measure)?;
        let sampler = IidContinuousLaplace::<T>::new(scale)?;

        Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::new_fallible(move |arg: &Vec<T>| sampler.sample_around(arg)),
            PrivacyMap::new_fallible(move |d_in: &QI| {
                let d_in = RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({d_in:?}) must be finite"))?;
                privacy_map.eval(&d_in)
            }),
        )
    }
}

#[proven(
    proof_path = "measurements/noise/nature/float/MakeNoise_VectorDomain_for_FloatExpFamily.tex"
)]
impl<T: Float, QI: Number, MO: 'static + Measure>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<2, QI>, MO> for FloatExpFamily<2>
where
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<2>: NoisePrivacyMap<LpDistance<2, RBig>, MO>,
{
    fn make_noise(
        self,
        (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<2, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LpDistance<2, QI>, MO, Vec<T>>> {
        if input_domain.element_domain.nan() {
            return fallible!(MakeMeasurement, "input_domain may not contain NaN elements");
        }

        let scale = RBig::try_from(self.scale)?;
        let distribution = ZExpFamily {
            scale: scale.clone(),
        };
        let output_measure = MO::default();
        let privacy_map =
            distribution.noise_privacy_map(&LpDistance::<2, RBig>::default(), &output_measure)?;
        let sampler = IidContinuousGaussian::<T>::new(scale)?;

        Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::new_fallible(move |arg: &Vec<T>| sampler.sample_around(arg)),
            PrivacyMap::new_fallible(move |d_in: &QI| {
                let d_in = RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({d_in:?}) must be finite"))?;
                privacy_map.eval(&d_in)
            }),
        )
    }
}

/// Float scalar mechanism
#[proven(
    proof_path = "measurements/noise/nature/float/MakeNoise_AtomDomain_for_FloatExpFamily.tex"
)]
impl<T: Float, QI: Number, MO: 'static + Measure> MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
    for FloatExpFamily<1>
where
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<1>: NoisePrivacyMap<LpDistance<1, RBig>, MO>,
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

#[proven(
    proof_path = "measurements/noise/nature/float/MakeNoise_AtomDomain_for_FloatExpFamily.tex"
)]
impl<T: Float, QI: Number, MO: 'static + Measure> MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO>
    for FloatExpFamily<2>
where
    RBig: TryFrom<T> + TryFrom<QI>,
    ZExpFamily<2>: NoisePrivacyMap<LpDistance<2, RBig>, MO>,
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
