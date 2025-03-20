use std::collections::HashMap;

use dashu::integer::{IBig, UBig};

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    measurements::{
        nature::{float::integerize_scale, integer::IntExpFamily},
        MakeNoiseThreshold, NoiseThresholdPrivacyMap, ZExpFamily,
    },
    metrics::{AbsoluteDistance, PartitionDistance},
    traits::{Hashable, Integer, SaturatingCast},
};

impl<MO: 'static + Measure, TK, TV, const P: usize, QI: Integer>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        PartitionDistance<AbsoluteDistance<QI>>,
        MO,
    > for IntExpFamily<P>
where
    TK: Hashable,
    TV: Integer + SaturatingCast<IBig>,
    IBig: From<TV>,
    UBig: TryFrom<QI>,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<PartitionDistance<AbsoluteDistance<UBig>>, MO>,
{
    type Threshold = TV;
    fn make_noise_threshold(
        self,
        (input_domain, input_metric): (
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            PartitionDistance<AbsoluteDistance<QI>>,
        ),
        threshold: TV,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            HashMap<TK, TV>,
            PartitionDistance<AbsoluteDistance<QI>>,
            MO,
        >,
    > {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
        };
        let threshold = IBig::from(threshold);

        let t_int = Transformation::new(
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
            input_metric.clone(),
            PartitionDistance::default(),
            StabilityMap::new_fallible(move |(l0, l1, li): &(u32, QI, QI)| {
                let l1 = UBig::try_from(l1.clone())
                    .map_err(|_| err!(FailedMap, "l1 ({:?}) must be non-negative", l1))?;

                let li = UBig::try_from(li.clone())
                    .map_err(|_| err!(FailedMap, "li ({:?}) must be non-negative", li))?;
                Ok((*l0, l1, li))
            }),
        )?;
        let m_noise = distribution.make_noise_threshold(t_int.output_space(), threshold)?;
        let f_native_int = Function::new(move |x: &HashMap<TK, IBig>| {
            x.iter()
                .map(|(k, v)| (k.clone(), TV::saturating_cast(v.clone())))
                .collect()
        });

        t_int >> m_noise >> f_native_int
    }
}
