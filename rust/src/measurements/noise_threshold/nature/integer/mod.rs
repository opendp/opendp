use std::collections::HashMap;

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};
use opendp_derive::proven;

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    measurements::{
        MakeNoiseThreshold, NoiseThresholdPrivacyMap, ZExpFamily,
        nature::{float::integerize_scale, integer::IntExpFamily},
    },
    metrics::{AbsoluteDistance, L0PInfDistance},
    traits::{Hashable, Integer, Number, SaturatingCast},
};

#[cfg(test)]
mod test;

impl<TK, TV, const P: usize, QI: Number, MO: 'static + Measure>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PInfDistance<P, AbsoluteDistance<QI>>,
        MO,
    > for IntExpFamily<P>
where
    TK: Hashable,
    TV: Integer + SaturatingCast<IBig>,
    IBig: From<TV>,
    RBig: TryFrom<QI>,
    UBig: TryFrom<TV>,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<L0PInfDistance<P, AbsoluteDistance<RBig>>, MO>,
{
    type Threshold = TV;
    fn make_noise_threshold(
        self,
        input_space: (
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            L0PInfDistance<P, AbsoluteDistance<QI>>,
        ),
        threshold: TV,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
            L0PInfDistance<P, AbsoluteDistance<QI>>,
            MO,
            HashMap<TK, TV>,
        >,
    > {
        let distribution = ZExpFamily {
            scale: integerize_scale(self.scale, 0)?,
        };

        let t_int = make_int_to_bigint_threshold::<TK, TV, P, QI>(input_space)?;
        let m_noise =
            distribution.make_noise_threshold(t_int.output_space(), IBig::from(threshold))?;
        let f_native_int = then_saturating_cast_hashmap();

        t_int >> m_noise >> f_native_int
    }
}

/// # Proof Definition
/// For any choice of arguments, returns a valid transformation.
#[proven(
    proof_path = "measurements/noise_threshold/nature/integer/make_int_to_bigint_threshold.tex"
)]
fn make_int_to_bigint_threshold<TK, TV, const P: usize, QI: Number>(
    input_space: (
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PInfDistance<P, AbsoluteDistance<QI>>,
    ),
) -> Fallible<
    Transformation<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PInfDistance<P, AbsoluteDistance<QI>>,
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        L0PInfDistance<P, AbsoluteDistance<RBig>>,
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
        input_metric,
        MapDomain {
            key_domain: input_domain.key_domain.clone(),
            value_domain: AtomDomain::<IBig>::default(),
        },
        L0PInfDistance::default(),
        Function::new(move |arg: &HashMap<TK, TV>| {
            arg.iter()
                .map(|(k, v)| (k.clone(), IBig::from(v.clone())))
                .collect()
        }),
        StabilityMap::new_fallible(move |(l0, lp, li): &(u32, QI, QI)| {
            let lp = RBig::try_from(lp.clone())
                .map_err(|_| err!(FailedMap, "l{P} ({lp:?}) must be finite"))?;

            let li = RBig::try_from(li.clone())
                .map_err(|_| err!(FailedMap, "li ({:?}) must be finite", li))?;
            Ok((*l0, lp, li))
        }),
    )
}

/// # Proof Definition
/// For any choice of arguments, returns a valid postprocessor.
#[proven(
    proof_path = "measurements/noise_threshold/nature/integer/then_saturating_cast_hashmap.tex"
)]
fn then_saturating_cast_hashmap<TK: Hashable, TV: SaturatingCast<IBig>>()
-> Function<HashMap<TK, IBig>, HashMap<TK, TV>> {
    Function::new(move |arg: &HashMap<TK, IBig>| {
        arg.iter()
            .map(|(k, v)| (k.clone(), TV::saturating_cast(v.clone())))
            .collect()
    })
}
