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
        nature::float::{
            find_nearest_multiple_of_2k, get_rounding_distance, integerize_scale, x_div_2k,
            x_mul_2k, FloatExpFamily,
        },
        MakeNoiseThreshold, NoiseThresholdPrivacyMap, ZExpFamily,
    },
    metrics::{AbsoluteDistance, PartitionDistance},
    traits::{CastInternalRational, ExactIntCast, Float, Hashable},
};

impl<MO: 'static + Measure, TK, TV, const P: usize, QI: Float>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        PartitionDistance<AbsoluteDistance<QI>>,
        MO,
    > for FloatExpFamily<P>
where
    TK: Hashable,
    TV: Float,
    RBig: TryFrom<TV> + TryFrom<QI>,
    i32: ExactIntCast<TV::Bits>,
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
        let FloatExpFamily { scale, k } = self;
        let distribution = ZExpFamily {
            scale: integerize_scale(scale, k)?,
        };
        let rounding_distance = get_rounding_distance::<TV>(k, None)?;

        let threshold = find_next_multiple_of_2k(
            RBig::try_from(threshold).map_err(|_| {
                err!(
                    MakeTransformation,
                    "threshold ({threshold}) must be non-negative"
                )
            })?,
            k,
        );

        let t_int = Transformation::new(
            input_domain.clone(),
            MapDomain {
                key_domain: input_domain.key_domain,
                value_domain: AtomDomain::<IBig>::default(),
            },
            Function::new(move |arg: &HashMap<TK, TV>| {
                (arg.clone().into_iter())
                    .map(|(key, val)| {
                        let val = RBig::try_from(val).unwrap_or(RBig::ZERO);
                        (key, find_nearest_multiple_of_2k(val, k))
                    })
                    .collect()
            }),
            input_metric.clone(),
            PartitionDistance::default(),
            StabilityMap::new_fallible(move |(l0, l1, li): &(u32, QI, QI)| {
                let l1 = RBig::try_from(l1.clone())
                    .map_err(|_| err!(FailedMap, "l1 ({:?}) must be finite", l1))?;
                let l1 =
                    find_next_multiple_of_2k(l1 + rounding_distance.clone() * RBig::from(*l0), k);

                let li = RBig::try_from(li.clone())
                    .map_err(|_| err!(FailedMap, "li ({:?}) must be finite", li))?;
                let li = find_next_multiple_of_2k(li + rounding_distance.clone(), k);
                Ok((*l0, l1.into_parts().1, li.into_parts().1))
            }),
        )?;
        let m_noise = distribution.make_noise_threshold(t_int.output_space(), threshold)?;
        t_int >> m_noise >> then_deintegerize_hashmap(k)
    }
}

pub fn then_deintegerize_hashmap<K: Hashable, V: Clone + CastInternalRational>(
    k: i32,
) -> Function<HashMap<K, IBig>, HashMap<K, V>> {
    Function::new(move |x: &HashMap<K, IBig>| {
        (x.clone().into_iter())
            .map(|(key, val)| (key, V::from_rational(x_mul_2k(val, k))))
            .collect()
    })
}

/// Find index of nearest multiple of $2^k$ from x.
///
/// # Proof Definition
/// For any setting of input arguments, return the integer $argmin_i |i 2^k - x|$.
fn find_next_multiple_of_2k(x: RBig, k: i32) -> IBig {
    // exactly compute x/2^k and break into fractional parts
    let (numer, denom) = x_div_2k(x, k).into_parts();

    let offset = denom.clone() * numer.sign();
    (numer + offset) / denom
}
