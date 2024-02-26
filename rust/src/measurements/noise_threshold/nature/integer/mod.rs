use std::collections::HashMap;

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    measurements::{
        MakeNoiseThreshold, NoiseThresholdPrivacyMap, ZExpFamily,
        nature::{float::integerize_scale, integer::IntExpFamily},
    },
    metrics::{AbsoluteDistance, L0PI},
    traits::{Hashable, Integer, SaturatingCast},
};

impl<MO: 'static + Measure, TK, TV, const P: usize, QI: Integer>
    MakeNoiseThreshold<MapDomain<AtomDomain<TK>, AtomDomain<TV>>, L0PI<P, AbsoluteDistance<QI>>, MO>
    for IntExpFamily<P>
where
    TK: Hashable,
    TV: Integer + SaturatingCast<IBig>,
    IBig: From<TV>,
    RBig: TryFrom<QI>,
    UBig: TryFrom<TV>,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<L0PI<P, AbsoluteDistance<RBig>>, MO>,
{
    type Threshold = TV;
    fn make_noise_threshold(
        self,
        input_space: (
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            L0PI<P, AbsoluteDistance<QI>>,
        ),
        threshold: TV,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            HashMap<TK, TV>,
            L0PI<P, AbsoluteDistance<QI>>,
            MO,
        >,
    > {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
        };
        let threshold = UBig::try_from(threshold).map_err(|_| {
            err!(
                MakeTransformation,
                "threshold ({threshold}) must be positive"
            )
        })?;

        let t_int = make_int_to_bigint_threshold::<TK, TV, P, QI>(input_space)?;
        let m_noise = distribution.make_noise_threshold(t_int.output_space(), threshold)?;
        let f_native_int = Function::new(move |x: &HashMap<TK, IBig>| {
            x.iter()
                .map(|(k, v)| (k.clone(), TV::saturating_cast(v.clone())))
                .collect()
        });

        t_int >> m_noise >> f_native_int
    }
}

fn make_int_to_bigint_threshold<TK, TV, const P: usize, QI: Integer>(
    input_space: (
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PI<P, AbsoluteDistance<QI>>,
    ),
) -> Fallible<
    Transformation<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        L0PI<P, AbsoluteDistance<QI>>,
        L0PI<P, AbsoluteDistance<RBig>>,
    >,
>
where
    TK: Hashable,
    TV: Integer + SaturatingCast<IBig>,
    IBig: From<TV>,
    RBig: TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;
    Transformation::new(
        input_domain.clone(),
        MapDomain {
            key_domain: input_domain.key_domain.clone(),
            value_domain: AtomDomain::<IBig>::default(),
        },
        Function::new(move |arg: &HashMap<TK, TV>| {
            arg.iter()
                .map(|(k, v)| (k.clone(), IBig::from(v.clone())))
                .collect()
        }),
        input_metric,
        L0PI::default(),
        StabilityMap::new_fallible(move |(l0, l1, li): &(u32, QI, QI)| {
            let l1 = RBig::try_from(l1.clone())
                .map_err(|_| err!(FailedMap, "l1 ({:?}) must be finite", l1))?;

            let li = RBig::try_from(li.clone())
                .map_err(|_| err!(FailedMap, "li ({:?}) must be finite", li))?;
            Ok((*l0, l1, li))
        }),
    )
}
