use std::collections::HashMap;

use dashu::{integer::IBig, rational::RBig};
use opendp_derive::proven;

use crate::{
    core::{Function, Measure, Measurement, StabilityMap, Transformation},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    measurements::{
        MakeNoiseThreshold, NoiseThresholdPrivacyMap, ZExpFamily,
        nature::float::{
            FloatExpFamily, find_nearest_multiple_of_2k, get_min_k, get_rounding_distance,
            integerize_scale, x_mul_2k,
        },
    },
    metrics::{AbsoluteDistance, L0PInfDistance},
    traits::{CastInternalRational, ExactIntCast, Float, Hashable, Number},
};

#[cfg(test)]
mod test;

#[proven(
    proof_path = "measurements/noise_threshold/nature/float/MakeNoiseThreshold_MapDomain_for_FloatExpFamily.tex"
)]
impl<TK, TV, const P: usize, QI: Number, MO: 'static + Measure>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PInfDistance<P, AbsoluteDistance<QI>>,
        MO,
    > for FloatExpFamily<P>
where
    TK: Hashable,
    TV: Float,
    RBig: TryFrom<TV> + TryFrom<QI>,
    i32: ExactIntCast<TV::Bits>,
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
        let FloatExpFamily { scale, k } = self;
        let distribution = ZExpFamily {
            scale: integerize_scale(scale, k)?,
        };

        if threshold.is_sign_negative() {
            return fallible!(
                FailedFunction,
                "threshold ({threshold}) must not be negative"
            );
        }

        let Ok(r_threshold) = RBig::try_from(threshold) else {
            return fallible!(FailedFunction, "threshold ({threshold}) must be finite");
        };

        let r_threshold = x_mul_2k(r_threshold, -k).round();

        let t_int = make_float_to_bigint_threshold::<TK, TV, P, QI>(input_space, threshold, k)?;
        let m_noise = distribution.make_noise_threshold(t_int.output_space(), r_threshold)?;
        t_int >> m_noise >> then_deintegerize_hashmap(k)?
    }
}

#[proven(
    proof_path = "measurements/noise_threshold/nature/float/make_float_to_bigint_threshold.tex"
)]
fn make_float_to_bigint_threshold<TK, TV, const P: usize, QI: Number>(
    (input_domain, input_metric): (
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        L0PInfDistance<P, AbsoluteDistance<QI>>,
    ),
    threshold: TV,
    k: i32,
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
    TV: Float,
    RBig: TryFrom<TV> + TryFrom<QI>,
    i32: ExactIntCast<TV::Bits>,
{
    if input_domain.value_domain.nan() {
        return fallible!(
            MakeTransformation,
            "input_domain hashmap values may not contain NaN elements"
        );
    }
    let Ok(r_threshold) = RBig::try_from(threshold) else {
        return fallible!(MakeTransformation, "threshold ({threshold}) must be finite");
    };
    let k_min = get_min_k::<TV>();
    if k < k_min {
        return fallible!(
            MakeTransformation,
            "k ({k}) must not be smaller than {k_min}"
        );
    }
    Transformation::new(
        input_domain.clone(),
        input_metric.clone(),
        MapDomain {
            key_domain: input_domain.key_domain,
            value_domain: AtomDomain::<IBig>::default(),
        },
        L0PInfDistance::default(),
        Function::new(move |arg: &HashMap<TK, TV>| {
            (arg.into_iter())
                .map(|(key, val)| {
                    let val = RBig::try_from(val.clamp(TV::MIN_FINITE, TV::MAX_FINITE))
                        .unwrap_or(RBig::ZERO);
                    (key.clone(), find_nearest_multiple_of_2k(val, k))
                })
                .collect()
        }),
        StabilityMap::new_fallible(move |(l0, lp, li): &(u32, QI, QI)| {
            // Lp sensitivity
            let r_lp = RBig::try_from(lp.clone())
                .map_err(|_| err!(FailedMap, "l{P} ({lp:?}) must be finite"))?;
            let r_lp_round = get_rounding_distance::<TV, P>(k, Some(*l0 as usize))?;
            let r_lp = x_mul_2k(r_lp + r_lp_round.clone(), -k);

            // L\infty sensitivity
            let r_li = RBig::try_from(li.clone())
                .map_err(|_| err!(FailedMap, "li ({li:?}) must be finite"))?;

            // this check is not necessary for stability,
            // but it gives a more understandable error message
            if r_li > r_threshold {
                return fallible!(
                    FailedMap,
                    "threshold ({threshold}) must not be smaller than l-infinity sensitivity ({li})"
                );
            }
            let r_li_round = get_rounding_distance::<TV, P>(k, Some(1))?;
            let r_li = x_mul_2k(r_li + r_li_round.clone(), -k);

            Ok((*l0, r_lp, r_li))
        }),
    )
}

#[proven(proof_path = "measurements/noise_threshold/nature/float/then_deintegerize_hashmap.tex")]
pub fn then_deintegerize_hashmap<K: Hashable, V: Clone + CastInternalRational>(
    k: i32,
) -> Fallible<Function<HashMap<K, IBig>, HashMap<K, V>>> {
    if k == i32::MIN {
        return fallible!(MakeTransformation, "k must not be i32::MIN");
    }
    Ok(Function::new(move |x: &HashMap<K, IBig>| {
        (x.clone().into_iter())
            .map(|(key, val)| (key, V::from_rational(x_mul_2k(RBig::from(val), k))))
            .collect()
    }))
}
