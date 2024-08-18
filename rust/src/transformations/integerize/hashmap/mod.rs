use std::collections::HashMap;

use dashu::{integer::IBig, rational::RBig};

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, MapDomain},
    error::Fallible,
    metrics::LpDistance,
    traits::{CastInternalRational, ExactIntCast, Float, Hashable, Number},
};

use super::{find_nearest_multiple_of_2k, get_rounding_distance, x_div_2k, x_mul_2k};

pub fn make_integerize_hashmap<KIA, TIA, const P: usize, QI>(
    input_space: (
        MapDomain<AtomDomain<KIA>, AtomDomain<TIA>>,
        LpDistance<P, QI>,
    ),
    k: i32,
) -> Fallible<
    Transformation<
        MapDomain<AtomDomain<KIA>, AtomDomain<TIA>>,
        MapDomain<AtomDomain<KIA>, AtomDomain<IBig>>,
        LpDistance<P, QI>,
        LpDistance<P, RBig>,
    >,
>
where
    KIA: Hashable,
    TIA: Float,
    QI: Number,
    i32: ExactIntCast<TIA::Bits>,
    RBig: TryFrom<TIA> + TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;

    let rounding_distance = get_rounding_distance::<TIA>(k, None)?;

    let output_domain = MapDomain {
        key_domain: input_domain.key_domain.clone(),
        value_domain: AtomDomain::<IBig>::default(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new(move |arg: &HashMap<KIA, TIA>| {
            (arg.clone().into_iter())
                .map(|(key, val)| {
                    let v = RBig::try_from(val).unwrap_or(RBig::ZERO);
                    (key, find_nearest_multiple_of_2k(v, k))
                })
                .collect()
        }),
        input_metric.clone(),
        LpDistance::default(),
        StabilityMap::new_fallible(move |d_in: &QI| {
            let d_in = RBig::try_from(d_in.clone())
                .map_err(|_| err!(FailedMap, "d_in ({:?}) must be finite", d_in))?;
            Ok(x_div_2k(d_in + rounding_distance.clone(), k))
        }),
    )
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
