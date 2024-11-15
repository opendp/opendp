use dashu::{
    integer::{fast_div::ConstDivisor, IBig},
    rational::RBig,
};

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::{MakeNoise, NoisePrivacyMap, ZExpFamily},
    metrics::{AbsoluteDistance, LpDistance, ModularMetric},
    traits::{Integer, Number, SaturatingCast},
    transformations::{make_vec, then_index_or_default},
};

use super::float::integerize_scale;

pub struct IntExpFamily<const P: usize> {
    pub scale: f64,
    pub divisor: Option<ConstDivisor>,
}

/// Integer vector mechanism
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
        (input_domain, input_metric): (VectorDomain<AtomDomain<T>>, LpDistance<P, QI>),
    ) -> Fallible<Measurement<VectorDomain<AtomDomain<T>>, Vec<T>, LpDistance<P, QI>, MO>> {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
            divisor: self.divisor,
        };
        let modular = input_metric.modular();

        let t_int = Transformation::new(
            input_domain.clone(),
            VectorDomain {
                element_domain: AtomDomain::<IBig>::default(),
                size: input_domain.size.clone(),
            },
            Function::new(move |arg: &Vec<T>| {
                arg.into_iter()
                    .cloned()
                    .map(|x_i| IBig::from(x_i.clone()))
                    .map(|x_i| {
                        if modular {
                            x_i + IBig::from(T::MIN_FINITE)
                        } else {
                            x_i
                        }
                    })
                    .collect()
            }),
            input_metric.clone(),
            LpDistance::default(),
            StabilityMap::new_fallible(move |d_in: &QI| {
                RBig::try_from(d_in.clone())
                    .map_err(|_| err!(FailedMap, "d_in ({:?}) must be finite", d_in))
            }),
        )?;
        let m_noise = distribution.make_noise(t_int.output_space())?;

        let f_native_int = Function::new(move |x: &Vec<IBig>| {
            x.into_iter()
                .cloned()
                .map(|x_i| {
                    if modular {
                        x_i - IBig::from(T::MIN_FINITE)
                    } else {
                        x_i
                    }
                })
                .map(T::saturating_cast)
                .collect()
        });

        t_int >> m_noise >> f_native_int
    }
}

/// Integer scalar mechanism
impl<MO: 'static + Measure, T: Integer, const P: usize, QI: Number>
    MakeNoise<AtomDomain<T>, AbsoluteDistance<QI>, MO> for IntExpFamily<P>
where
    T: Integer + SaturatingCast<IBig>,
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
