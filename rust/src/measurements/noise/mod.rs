use dashu::{integer::IBig, rational::RBig};
use opendp_derive::bootstrap;

use crate::{
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, LpDistance},
    traits::{
        samplers::{sample_discrete_gaussian, sample_discrete_laplace},
        ExactIntCast, Float, FloatBits, Integer, Number, SaturatingCast,
    },
    transformations::{
        get_min_k, integerize_scale, make_big_int, make_integerize_vec, make_vec,
        then_deintegerize_vec, then_index, then_native_int,
    },
};

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(D(suppress), QI(suppress))
)]
/// Make a Measurement that adds noise from a distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`          |
/// | ------------------------------- | ------------ | ----------------------- |
/// | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be privatized.
/// * `input_metric` - Metric of the data type to be privatized.
/// * `scale` - Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `DI` - Domain of the data to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
/// * `MI` - Input Metric to measure distances between members of the input domain.
/// * `RV` - Noise distribution to be added to the dataset.
/// * `MO` - Output Measure.
pub fn make_noise<DI: Domain, MI: Metric, RV, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    distribution: RV,
) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>
where
    ((DI, MI), RV): MakeNoise<DI, MI, RV, MO>,
    (DI, MI): MetricSpace,
{
    ((input_domain, input_metric), distribution).make_noise()
}

/// Common interface for noise perturbation mechanisms.
pub trait MakeNoise<DI: Domain, MI: Metric, RV, MO: Measure>
where
    (DI, MI): MetricSpace,
{
    /// # Proof Definition
    /// For any choice of arguments to `self`,
    /// returns a valid measurement or error.
    fn make_noise(self) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>;
}

/// Common interface for privacy maps.
pub trait NoisePrivacyMap<MI: Metric, MO: Measure, RV> {
    fn privacy_map(distribution: RV) -> Fallible<PrivacyMap<MI, MO>>;
}

#[derive(Clone)]
pub struct ZExpFamily<const P: usize> {
    pub scale: RBig,
}

pub trait Sample: 'static + Clone + Send + Sync {
    fn sample(self) -> Fallible<IBig>;
}

impl Sample for ZExpFamily<1> {
    fn sample(self) -> Fallible<IBig> {
        sample_discrete_laplace(self.scale)
    }
}

impl Sample for ZExpFamily<2> {
    fn sample(self) -> Fallible<IBig> {
        sample_discrete_gaussian(self.scale)
    }
}

/// Big integer vector mechanism
impl<MI: Metric, MO: 'static + Measure, RV: Sample>
    MakeNoise<VectorDomain<AtomDomain<IBig>>, MI, RV, MO>
    for ((VectorDomain<AtomDomain<IBig>>, MI), RV)
where
    (VectorDomain<AtomDomain<IBig>>, MI): MetricSpace,
    ((MI, MO), RV): NoisePrivacyMap<MI, MO, RV>,
{
    /// Make a Measurement that adds noise from the discrete distribution RV to each value in the input.
    fn make_noise(
        self,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<IBig>>, Vec<IBig>, MI, MO>> {
        let ((input_domain, input_metric), distribution) = self;
        Measurement::new(
            input_domain,
            Function::new_fallible(enclose!(distribution, move |x: &Vec<IBig>| {
                x.into_iter()
                    .cloned()
                    .map(|x_i| RV::sample(distribution.clone()).map(|s| x_i + s))
                    .collect()
            })),
            input_metric,
            MO::default(),
            <((MI, MO), RV)>::privacy_map(distribution)?,
        )
    }
}

// # FLOATING-POINT MECHANISMS

pub struct FloatExpFamily<const P: usize> {
    pub scale: f64,
    pub k: i32,
}

/// Float vector mechanism
impl<MO: 'static + Measure, T: Float, const P: usize, QI: Number>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, FloatExpFamily<P>, MO>
    for (
        (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
        FloatExpFamily<P>,
    )
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ((LpDistance<P, RBig>, MO), ZExpFamily<P>):
        NoisePrivacyMap<LpDistance<P, RBig>, MO, ZExpFamily<P>>,
    ZExpFamily<P>: Sample,
{
    fn make_noise(
        self,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, LpDistance<P, QI>, MO>> {
        let (input_space, distribution) = self;
        let k = distribution.k;

        let distribution = ZExpFamily {
            scale: integerize_scale(distribution.scale, distribution.k)?,
        };

        let t_int = make_integerize_vec(input_space, k)?;
        let m_noise = (t_int.output_space(), distribution).make_noise()?;

        t_int >> m_noise >> then_deintegerize_vec(k)
    }
}

/// Float scalar mechanism
impl<MO: 'static + Measure, T: Float, const P: usize, QI: Number>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, FloatExpFamily<P>, MO>
    for ((AtomDomain<T>, AbsoluteDistance<QI>), FloatExpFamily<P>)
where
    i32: ExactIntCast<<T as FloatBits>::Bits>,
    RBig: TryFrom<T> + TryFrom<QI>,
    ((LpDistance<P, RBig>, MO), ZExpFamily<P>):
        NoisePrivacyMap<LpDistance<P, RBig>, MO, ZExpFamily<P>>,
    ZExpFamily<P>: Sample,
{
    fn make_noise(self) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<QI>, MO>> {
        let (input_space, distribution) = self;
        let t_vec = make_vec(input_space)?;
        let m_noise = (t_vec.output_space(), distribution).make_noise()?;

        t_vec >> m_noise >> then_index(0)
    }
}

// # NATIVE-INTEGER MECHANISMS

pub struct IntExpFamily<const P: usize> {
    pub scale: f64,
}

/// Integer vector mechanism
impl<MO: 'static + Measure, T: Integer, const P: usize, QI: Number>
    MakeNoise<VectorDomain<AtomDomain<T>>, LpDistance<P, QI>, IntExpFamily<P>, MO>
    for (
        (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
        IntExpFamily<P>,
    )
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    RBig: TryFrom<QI>,
    ((LpDistance<P, RBig>, MO), ZExpFamily<P>):
        NoisePrivacyMap<LpDistance<P, RBig>, MO, ZExpFamily<P>>,
    ZExpFamily<P>: Sample,
{
    fn make_noise(
        self,
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, LpDistance<P, QI>, MO>> {
        let (input_space, distribution) = self;

        let distribution = ZExpFamily {
            scale: integerize_scale(distribution.scale, 0)?,
        };

        let t_int = make_big_int(input_space)?;
        let m_noise = (t_int.output_space(), distribution).make_noise()?;

        t_int >> m_noise >> then_native_int()
    }
}

/// Integer scalar mechanism
impl<MO: 'static + Measure, T: Integer, const P: usize, QI: Number>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, IntExpFamily<P>, MO>
    for ((AtomDomain<T>, AbsoluteDistance<QI>), IntExpFamily<P>)
where
    T: Integer + SaturatingCast<IBig>,
    IBig: From<T>,
    RBig: TryFrom<QI>,
    ((LpDistance<P, RBig>, MO), ZExpFamily<P>):
        NoisePrivacyMap<LpDistance<P, RBig>, MO, ZExpFamily<P>>,
    ZExpFamily<P>: Sample,
{
    fn make_noise(self) -> Fallible<Measurement<AtomDomain<T>, T, AbsoluteDistance<QI>, MO>> {
        let (input_space, distribution) = self;
        let t_vec = make_vec(input_space)?;
        let m_noise = (t_vec.output_space(), distribution).make_noise()?;

        t_vec >> m_noise >> then_index(0)
    }
}


// # UTILITIES FOR SHORTHAND CONSTRUCTOR FUNCTIONS
// * make_gaussian
// * make_laplace
// * make_geometric

pub trait NoiseDomain: Domain {
    type Atom;
}

impl<T: Number> NoiseDomain for AtomDomain<T> {
    type Atom = T;
}

impl<T: Number> NoiseDomain for VectorDomain<AtomDomain<T>> {
    type Atom = T;
}

pub trait Nature<const P: usize> {
    type Dist;
    fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::Dist>;
}

macro_rules! impl_Nature_float {
    ($($T:ty)+) => ($(impl<const P: usize> Nature<P> for $T {
        type Dist = FloatExpFamily<P>;
        fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::Dist> {
            Ok(FloatExpFamily::<P> {
                scale,
                k: k.unwrap_or_else(get_min_k::<$T>),
            })
        }
    })+)
}
macro_rules! impl_Nature_int {
    ($($T:ty)+) => ($(impl<const P: usize> Nature<P> for $T {
        type Dist = IntExpFamily<P>;
        fn new_distribution(scale: f64, k: Option<i32>) -> Fallible<Self::Dist> {
            if k.unwrap_or(0) != 0 {
                return fallible!(MakeMeasurement, "k is only valid for domains over floats");
            }
            Ok(IntExpFamily::<P> {
                scale,
            })
        }
    })+)
}

impl_Nature_float!(f32 f64);
impl_Nature_int!(i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize);
