use dashu::{integer::IBig, rational::RBig};
use opendp_derive::bootstrap;

use crate::{
    core::{
        Domain, Function, Measurement, Metric, MetricSpace, PrivacyMap, StabilityMap,
        Transformation,
    },
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, Sample},
    measures::MaxDivergence,
    metrics::{AbsoluteDistance, LInfDistance, LpDistance},
    traits::{
        Float, InfCast, Integer, Number, SaturatingCast,
        samplers::{
            ContinuousVectorL1Staircase, ContinuousVectorLinfStaircase, JointDiscreteL1Staircase,
            JointDiscreteLinfStaircase, RoundedContinuousVectorSampler,
        },
    },
    transformations::{make_vec, then_index_or_default},
};

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[bootstrap(features("contrib"), generics(DI(suppress), MI(suppress)))]
/// Make a vector-valued L1 staircase mechanism.
///
/// Floating-point carriers use the continuous sampler. Integer carriers use the
/// discrete lattice sampler.
pub fn make_l1_staircase<DI: Domain, MI: Metric>(
    input_domain: DI,
    input_metric: MI,
    delta: f64,
    r: f64,
    epsilon: f64,
) -> Fallible<Measurement<DI, MI, MaxDivergence, DI::Carrier>>
where
    L1Staircase: MakeNoise<DI, MI, MaxDivergence>,
    (DI, MI): MetricSpace,
{
    L1Staircase { delta, r, epsilon }.make_noise((input_domain, input_metric))
}

#[bootstrap(features("contrib"), generics(DI(suppress), MI(suppress)))]
/// Make a vector-valued L-infinity staircase mechanism.
///
/// Floating-point carriers use the continuous sampler. Integer carriers use the
/// discrete lattice sampler.
pub fn make_linf_staircase<DI: Domain, MI: Metric>(
    input_domain: DI,
    input_metric: MI,
    delta: f64,
    r: f64,
    epsilon: f64,
) -> Fallible<Measurement<DI, MI, MaxDivergence, DI::Carrier>>
where
    LinfStaircase: MakeNoise<DI, MI, MaxDivergence>,
    (DI, MI): MetricSpace,
{
    LinfStaircase { delta, r, epsilon }.make_noise((input_domain, input_metric))
}

#[derive(Clone)]
pub struct L1Staircase {
    delta: f64,
    r: f64,
    epsilon: f64,
}

#[derive(Clone)]
pub struct LinfStaircase {
    delta: f64,
    r: f64,
    epsilon: f64,
}

impl L1Staircase {
    fn discrete_params(&self) -> Fallible<(usize, usize, RBig)> {
        discrete_params(self.delta, self.r, self.epsilon)
    }
}

impl LinfStaircase {
    fn discrete_params(&self) -> Fallible<(usize, usize, RBig)> {
        discrete_params(self.delta, self.r, self.epsilon)
    }
}

fn make_l1_float_vec<T, QI>(
    mech: L1Staircase,
    (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<1, QI>),
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LpDistance<1, QI>, MaxDivergence, Vec<T>>>
where
    T: Float,
    QI: Number,
    RBig: TryFrom<T> + TryFrom<QI>,
{
    if input_domain.element_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain may not contain NaN elements");
    }

    let delta = rbig_from_f64("delta", mech.delta)?;
    let r = rbig_from_f64("r", mech.r)?;
    let epsilon = rbig_from_f64("epsilon", mech.epsilon)?;
    let sampler_delta = delta.clone();

    Measurement::new(
        input_domain,
        input_metric,
        MaxDivergence::default(),
        Function::new_fallible(move |arg: &Vec<T>| {
            ContinuousVectorL1Staircase::new(
                arg.len(),
                sampler_delta.clone(),
                r.clone(),
                epsilon.clone(),
            )?
            .sample_around(arg)
        }),
        staircase_privacy_map::<LpDistance<1, QI>, QI>(delta, mech.epsilon),
    )
}

fn make_linf_float_vec<T>(
    mech: LinfStaircase,
    (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LInfDistance<T>),
) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, LInfDistance<T>, MaxDivergence, Vec<T>>>
where
    T: Float,
    RBig: TryFrom<T>,
{
    if input_domain.element_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain may not contain NaN elements");
    }

    let delta = rbig_from_f64("delta", mech.delta)?;
    let r = rbig_from_f64("r", mech.r)?;
    let epsilon = rbig_from_f64("epsilon", mech.epsilon)?;
    let sampler_delta = delta.clone();

    Measurement::new(
        input_domain,
        input_metric,
        MaxDivergence::default(),
        Function::new_fallible(move |arg: &Vec<T>| {
            ContinuousVectorLinfStaircase::new(
                arg.len(),
                sampler_delta.clone(),
                r.clone(),
                epsilon.clone(),
            )?
            .sample_around(arg)
        }),
        staircase_privacy_map::<LInfDistance<T>, T>(delta, mech.epsilon),
    )
}

fn make_linf_float_atom<T, QI>(
    mech: LinfStaircase,
    input_space: (AtomDomain<T>, AbsoluteDistance<QI>),
) -> Fallible<Measurement<AtomDomain<T>, AbsoluteDistance<QI>, MaxDivergence, T>>
where
    T: Float,
    QI: Number,
    RBig: TryFrom<T> + TryFrom<QI>,
{
    let delta = rbig_from_f64("delta", mech.delta)?;
    let r = rbig_from_f64("r", mech.r)?;
    let epsilon = rbig_from_f64("epsilon", mech.epsilon)?;
    let sampler = ContinuousVectorLinfStaircase::<T>::new(1, delta.clone(), r, epsilon)?;

    Measurement::new(
        input_space.0,
        input_space.1,
        MaxDivergence::default(),
        Function::new_fallible(move |arg: &T| {
            Ok(sampler
                .sample_around(std::slice::from_ref(arg))?
                .into_iter()
                .next()
                .unwrap_or_default())
        }),
        staircase_privacy_map::<AbsoluteDistance<QI>, QI>(delta, mech.epsilon),
    )
}

macro_rules! impl_float_staircase {
    ($($T:ty),+) => {$(
        impl<QI> MakeNoise<VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>, MaxDivergence>
            for L1Staircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>),
            ) -> Fallible<
                Measurement<VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>, MaxDivergence, Vec<$T>>,
            > {
                make_l1_float_vec::<$T, QI>(self, input_space)
            }
        }

        impl<QI> MakeNoise<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence> for L1Staircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (AtomDomain<$T>, AbsoluteDistance<QI>),
            ) -> Fallible<Measurement<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence, $T>> {
                let t_vec = make_vec(input_space)?;
                let m_noise = self.make_noise(t_vec.output_space())?;
                t_vec >> m_noise >> then_index_or_default(0)
            }
        }

        impl MakeNoise<VectorDomain<AtomDomain<$T>>, LInfDistance<$T>, MaxDivergence>
            for LinfStaircase
        {
            fn make_noise(
                self,
                input_space: (VectorDomain<AtomDomain<$T>>, LInfDistance<$T>),
            ) -> Fallible<
                Measurement<VectorDomain<AtomDomain<$T>>, LInfDistance<$T>, MaxDivergence, Vec<$T>>,
            > {
                make_linf_float_vec::<$T>(self, input_space)
            }
        }

        impl<QI> MakeNoise<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence> for LinfStaircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (AtomDomain<$T>, AbsoluteDistance<QI>),
            ) -> Fallible<Measurement<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence, $T>> {
                make_linf_float_atom::<$T, QI>(self, input_space)
            }
        }
    )+}
}

impl_float_staircase!(f32, f64);

impl Sample for L1Staircase {
    fn sample(&self, shift: &Vec<IBig>) -> Fallible<Vec<IBig>> {
        let (delta, r, epsilon) = self.discrete_params()?;
        let noise =
            JointDiscreteL1Staircase::new(shift.len(), delta, r, epsilon)?.sample_noise()?;
        Ok(shift
            .iter()
            .cloned()
            .zip(noise)
            .map(|(x, z)| x + z)
            .collect())
    }
}

impl NoisePrivacyMap<LpDistance<1, RBig>, MaxDivergence> for L1Staircase {
    fn noise_privacy_map(
        &self,
        _input_metric: &LpDistance<1, RBig>,
        _output_measure: &MaxDivergence,
    ) -> Fallible<PrivacyMap<LpDistance<1, RBig>, MaxDivergence>> {
        let (delta, r, epsilon) = self.discrete_params()?;
        JointDiscreteL1Staircase::new(1, delta, r, epsilon)?;
        Ok(staircase_privacy_map::<LpDistance<1, RBig>, RBig>(
            RBig::from(delta),
            self.epsilon,
        ))
    }
}

impl MakeNoise<AtomDomain<IBig>, AbsoluteDistance<RBig>, MaxDivergence> for L1Staircase {
    fn make_noise(
        self,
        input_space: (AtomDomain<IBig>, AbsoluteDistance<RBig>),
    ) -> Fallible<Measurement<AtomDomain<IBig>, AbsoluteDistance<RBig>, MaxDivergence, IBig>> {
        let t_vec = make_vec(input_space)?;
        let m_noise = self.make_noise(t_vec.output_space())?;
        t_vec >> m_noise >> then_index_or_default(0)
    }
}

macro_rules! impl_integer_staircase {
    ($($T:ty),+) => {$(
        impl<QI> MakeNoise<VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>, MaxDivergence>
            for L1Staircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>),
            ) -> Fallible<
                Measurement<VectorDomain<AtomDomain<$T>>, LpDistance<1, QI>, MaxDivergence, Vec<$T>>,
            > {
                let t_int = make_int_to_bigint_l1(input_space)?;
                let m_noise = self.make_noise(t_int.output_space())?;
                t_int >> m_noise >> then_saturating_cast()
            }
        }

        impl<QI> MakeNoise<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence> for L1Staircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (AtomDomain<$T>, AbsoluteDistance<QI>),
            ) -> Fallible<Measurement<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence, $T>> {
                let t_vec = make_vec(input_space)?;
                let m_noise = self.make_noise(t_vec.output_space())?;
                t_vec >> m_noise >> then_index_or_default(0)
            }
        }

        impl MakeNoise<VectorDomain<AtomDomain<$T>>, LInfDistance<$T>, MaxDivergence>
            for LinfStaircase
        {
            fn make_noise(
                self,
                input_space: (VectorDomain<AtomDomain<$T>>, LInfDistance<$T>),
            ) -> Fallible<
                Measurement<VectorDomain<AtomDomain<$T>>, LInfDistance<$T>, MaxDivergence, Vec<$T>>,
            > {
                let t_int = make_int_to_bigint_linf(input_space)?;
                let m_noise = self.make_noise(t_int.output_space())?;
                t_int >> m_noise >> then_saturating_cast()
            }
        }

        impl<QI> MakeNoise<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence> for LinfStaircase
        where
            QI: Number,
            RBig: TryFrom<QI>,
        {
            fn make_noise(
                self,
                input_space: (AtomDomain<$T>, AbsoluteDistance<QI>),
            ) -> Fallible<Measurement<AtomDomain<$T>, AbsoluteDistance<QI>, MaxDivergence, $T>> {
                let t_vec = make_vec(input_space)?;
                let m_noise = L1Staircase {
                    delta: self.delta,
                    r: self.r,
                    epsilon: self.epsilon,
                }
                .make_noise(t_vec.output_space())?;
                t_vec >> m_noise >> then_index_or_default(0)
            }
        }
    )+}
}

impl_integer_staircase!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl Sample for LinfStaircase {
    fn sample(&self, shift: &Vec<IBig>) -> Fallible<Vec<IBig>> {
        let (delta, r, epsilon) = self.discrete_params()?;
        let noise =
            JointDiscreteLinfStaircase::new(shift.len(), delta, r, epsilon)?.sample_noise()?;
        Ok(shift
            .iter()
            .cloned()
            .zip(noise)
            .map(|(x, z)| x + z)
            .collect())
    }
}

impl NoisePrivacyMap<LInfDistance<IBig>, MaxDivergence> for LinfStaircase {
    fn noise_privacy_map(
        &self,
        _input_metric: &LInfDistance<IBig>,
        _output_measure: &MaxDivergence,
    ) -> Fallible<PrivacyMap<LInfDistance<IBig>, MaxDivergence>> {
        let (delta, r, epsilon) = self.discrete_params()?;
        JointDiscreteLinfStaircase::new(1, delta, r, epsilon)?;
        Ok(staircase_privacy_map::<LInfDistance<IBig>, IBig>(
            RBig::from(delta),
            self.epsilon,
        ))
    }
}

impl MakeNoise<AtomDomain<IBig>, AbsoluteDistance<RBig>, MaxDivergence> for LinfStaircase {
    fn make_noise(
        self,
        input_space: (AtomDomain<IBig>, AbsoluteDistance<RBig>),
    ) -> Fallible<Measurement<AtomDomain<IBig>, AbsoluteDistance<RBig>, MaxDivergence, IBig>> {
        let t_vec = make_vec(input_space)?;
        let m_noise = L1Staircase {
            delta: self.delta,
            r: self.r,
            epsilon: self.epsilon,
        }
        .make_noise(t_vec.output_space())?;
        t_vec >> m_noise >> then_index_or_default(0)
    }
}

fn rbig_from_f64(name: &str, value: f64) -> Fallible<RBig> {
    <RBig as TryFrom<f64>>::try_from(value)
        .map_err(|_| err!(MakeMeasurement, "{name} must be finite"))
}

fn discrete_params(delta: f64, r: f64, epsilon: f64) -> Fallible<(usize, usize, RBig)> {
    Ok((
        usize_from_integer_f64("delta", delta)?,
        usize_from_integer_f64("r", r)?,
        rbig_from_f64("epsilon", epsilon)?,
    ))
}

fn usize_from_integer_f64(name: &str, value: f64) -> Fallible<usize> {
    let rational = rbig_from_f64(name, value)?;
    let (num, den) = rational.into_parts();
    let (sign, mag) = num.into_parts();
    if !den.is_one() || sign == dashu::base::Sign::Negative {
        return fallible!(MakeMeasurement, "{name} must be a nonnegative integer");
    }
    usize::try_from(mag).map_err(|_| err!(MakeMeasurement, "{name} is too large"))
}

fn staircase_privacy_map<MI, QI>(delta: RBig, epsilon: f64) -> PrivacyMap<MI, MaxDivergence>
where
    MI: Metric<Distance = QI>,
    QI: Clone + 'static,
    RBig: TryFrom<QI>,
{
    PrivacyMap::new_fallible(move |d_in: &QI| {
        let d_in = RBig::try_from(d_in.clone())
            .map_err(|_| err!(FailedMap, "d_in must be finite and nonnegative"))?;
        if d_in < RBig::ZERO {
            return fallible!(FailedMap, "d_in must be nonnegative");
        }
        if delta <= RBig::ZERO {
            return fallible!(FailedMap, "delta must be positive");
        }
        let groups = f64::inf_cast(d_in / delta.clone())?.ceil();
        Ok(groups * epsilon)
    })
}

fn make_int_to_bigint_l1<T: Integer, QI: Number>(
    (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<1, QI>),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        LpDistance<1, QI>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<1, RBig>,
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
            size: input_domain.size,
        },
        LpDistance::default(),
        Function::new(move |x: &Vec<T>| x.iter().cloned().map(IBig::from).collect()),
        StabilityMap::new_fallible(move |d_in: &QI| {
            RBig::try_from(d_in.clone())
                .map_err(|_| err!(FailedMap, "d_in ({d_in:?}) must be finite"))
        }),
    )
}

fn make_int_to_bigint_linf<T: Integer>(
    (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LInfDistance<T>),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<T>>,
        LInfDistance<T>,
        VectorDomain<AtomDomain<IBig>>,
        LInfDistance<IBig>,
    >,
>
where
    IBig: From<T>,
{
    Transformation::new(
        input_domain.clone(),
        input_metric,
        VectorDomain {
            element_domain: AtomDomain::<IBig>::default(),
            size: input_domain.size,
        },
        LInfDistance::default(),
        Function::new(move |x: &Vec<T>| x.iter().cloned().map(IBig::from).collect()),
        StabilityMap::new_fallible(move |d_in: &T| Ok(IBig::from(d_in.clone()))),
    )
}

fn then_saturating_cast<TO: SaturatingCast<IBig>>() -> Function<Vec<IBig>, Vec<TO>> {
    Function::new(move |x: &Vec<IBig>| x.iter().cloned().map(TO::saturating_cast).collect())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        metrics::LpDistance,
        traits::samplers::test::{check_chi_square, check_kolmogorov_smirnov},
    };
    use dashu::rbig;

    #[test]
    fn test_make_l1_staircase_float() -> Fallible<()> {
        let meas = make_l1_staircase(
            VectorDomain::new(AtomDomain::<f64>::new_non_nan()),
            LpDistance::<1, f64>::default(),
            1.0,
            0.5,
            1.0,
        )?;
        let out = meas.invoke(&vec![0.0, 1.0])?;
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|x| x.is_finite()));
        Ok(())
    }

    #[test]
    fn test_make_linf_staircase_float() -> Fallible<()> {
        let meas = make_linf_staircase(
            VectorDomain::new(AtomDomain::<f64>::new_non_nan()),
            LInfDistance::<f64>::default(),
            1.0,
            0.5,
            1.0,
        )?;
        let out = meas.invoke(&vec![0.0, 1.0])?;
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|x| x.is_finite()));
        Ok(())
    }

    #[test]
    fn test_make_l1_staircase_integer() -> Fallible<()> {
        let meas = make_l1_staircase(
            VectorDomain::new(AtomDomain::<i32>::default()).with_size(2),
            LpDistance::<1, i32>::default(),
            2.0,
            1.0,
            1.0,
        )?;
        let out = meas.invoke(&vec![0, 1])?;
        assert_eq!(out.len(), 2);
        Ok(())
    }

    #[test]
    fn test_make_linf_staircase_integer() -> Fallible<()> {
        let meas = make_linf_staircase(
            VectorDomain::new(AtomDomain::<i32>::default()).with_size(2),
            LInfDistance::<i32>::default(),
            2.0,
            1.0,
            1.0,
        )?;
        let out = meas.invoke(&vec![0, 1])?;
        assert_eq!(out.len(), 2);
        Ok(())
    }

    #[test]
    fn test_continuous_l1_staircase_marginal_kolmogorov_smirnov() -> Fallible<()> {
        let sampler = ContinuousVectorL1Staircase::<f64>::new(2, rbig!(1), rbig!(1), rbig!(1))?;
        let mut samples = [0.0; 5000];
        for sample in samples.iter_mut() {
            *sample = sampler.sample_around(&[0.0, 0.0])?[0];
        }

        check_kolmogorov_smirnov(samples, |x| {
            continuous_staircase_marginal_cdf(x, UnitBall::L1)
        })
    }

    #[test]
    fn test_continuous_linf_staircase_marginal_kolmogorov_smirnov() -> Fallible<()> {
        let sampler = ContinuousVectorLinfStaircase::<f64>::new(2, rbig!(1), rbig!(1), rbig!(1))?;
        let mut samples = [0.0; 5000];
        for sample in samples.iter_mut() {
            *sample = sampler.sample_around(&[0.0, 0.0])?[0];
        }

        check_kolmogorov_smirnov(samples, |x| {
            continuous_staircase_marginal_cdf(x, UnitBall::Linf)
        })
    }

    #[test]
    fn test_discrete_l1_staircase_noise_chi_square() -> Fallible<()> {
        let sampler = JointDiscreteL1Staircase::new(2, 2, 1, rbig!(1))?;
        check_discrete_staircase_noise(
            || sampler.sample_noise(),
            lattice_points_l1(5),
            |point| point.iter().map(|x| x.unsigned_abs() as usize).sum(),
            |n| {
                if n == 0 { 1.0 } else { 4.0 * n as f64 }
            },
        )
    }

    #[test]
    fn test_discrete_linf_staircase_noise_chi_square() -> Fallible<()> {
        let sampler = JointDiscreteLinfStaircase::new(2, 2, 1, rbig!(1))?;
        check_discrete_staircase_noise(
            || sampler.sample_noise(),
            lattice_points_linf(4),
            |point| {
                point
                    .iter()
                    .map(|x| x.unsigned_abs() as usize)
                    .max()
                    .unwrap()
            },
            |n| {
                if n == 0 { 1.0 } else { 8.0 * n as f64 }
            },
        )
    }

    #[derive(Clone, Copy)]
    enum UnitBall {
        L1,
        Linf,
    }

    fn continuous_staircase_marginal_cdf(x: f64, unit_ball: UnitBall) -> f64 {
        const D: i32 = 2;
        const TERMS: usize = 200;
        let x_abs = x.abs();
        let b = (-1.0f64).exp();
        let mut norm = 0.0;
        let mut cdf_abs = 0.0;

        for i in 0..TERMS {
            let scale = (i + 1) as f64;
            let weight = scale.powi(D) * b.powi(i as i32);
            norm += weight;

            let conditional = if x_abs >= scale {
                1.0
            } else {
                match unit_ball {
                    UnitBall::L1 => 0.5 + x_abs / scale - x_abs * x_abs / (2.0 * scale * scale),
                    UnitBall::Linf => 0.5 + x_abs / (2.0 * scale),
                }
            };
            cdf_abs += weight * conditional;
        }

        let cdf_abs = cdf_abs / norm;
        if x < 0.0 { 1.0 - cdf_abs } else { cdf_abs }
    }

    fn check_discrete_staircase_noise(
        mut sample_noise: impl FnMut() -> Fallible<Vec<IBig>>,
        points: Vec<Vec<i32>>,
        norm: impl Fn(&[i32]) -> usize,
        shell_count: impl Fn(usize) -> f64,
    ) -> Fallible<()> {
        const N: usize = 60_000;
        const DELTA: usize = 2;
        const R: usize = 1;
        const TAIL_TERMS: usize = 200;

        let mut observed = vec![0u64; points.len() + 1];
        for _ in 0..N {
            let noise = sample_noise()?;
            let point = noise
                .into_iter()
                .map(i32::try_from)
                .collect::<Result<Vec<_>, _>>();

            let idx = point
                .ok()
                .and_then(|point| points.iter().position(|candidate| candidate == &point))
                .unwrap_or(points.len());
            observed[idx] += 1;
        }

        let normalizer: f64 = (0..TAIL_TERMS)
            .map(|n| shell_count(n) * staircase_point_weight(n, DELTA, R))
            .sum();

        let mut expected = points
            .iter()
            .map(|point| N as f64 * staircase_point_weight(norm(point), DELTA, R) / normalizer)
            .collect::<Vec<_>>();

        let point_mass: f64 = expected.iter().sum();
        expected.push(N as f64 - point_mass);

        check_chi_square(&observed, &expected)
    }

    fn staircase_point_weight(radius: usize, delta: usize, r: usize) -> f64 {
        let exponent = if radius == 0 {
            0
        } else {
            let k = radius / delta;
            let j = radius % delta;
            k + usize::from(j >= r)
        };
        (-(exponent as f64)).exp()
    }

    fn lattice_points_l1(max_radius: i32) -> Vec<Vec<i32>> {
        let mut points = (-max_radius..=max_radius)
            .flat_map(|x| {
                (-max_radius..=max_radius)
                    .filter_map(move |y| (x.abs() + y.abs() <= max_radius).then_some(vec![x, y]))
            })
            .collect::<Vec<_>>();
        points.sort();
        points
    }

    fn lattice_points_linf(max_radius: i32) -> Vec<Vec<i32>> {
        let mut points = (-max_radius..=max_radius)
            .flat_map(|x| {
                (-max_radius..=max_radius)
                    .filter_map(move |y| (x.abs().max(y.abs()) <= max_radius).then_some(vec![x, y]))
            })
            .collect::<Vec<_>>();
        points.sort();
        points
    }
}
