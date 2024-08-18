use dashu::{integer::IBig, rational::RBig};

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LpDistance,
    traits::{Integer, Number, SaturatingCast},
};

pub fn make_big_int<TIA, const P: usize, QI>(
    input_space: (VectorDomain<AtomDomain<TIA>>, LpDistance<P, QI>),
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<P, QI>,
        LpDistance<P, RBig>,
    >,
>
where
    TIA: Integer,
    QI: Number,
    IBig: From<TIA>,
    RBig: TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;

    let output_domain = VectorDomain {
        element_domain: AtomDomain::<IBig>::default(),
        size: input_domain.size.clone(),
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new(move |arg: &Vec<TIA>| {
            arg.into_iter()
                .cloned()
                .map(|x_i| IBig::from(x_i.clone()))
                .collect()
        }),
        input_metric.clone(),
        LpDistance::default(),
        StabilityMap::new_fallible(move |d_in: &QI| {
            RBig::try_from(d_in.clone())
                .map_err(|_| err!(FailedMap, "d_in ({:?}) must be finite", d_in))
        }),
    )
}

pub fn then_native_int<TO: SaturatingCast<IBig>>() -> Function<Vec<IBig>, Vec<TO>> {
    Function::new(move |x: &Vec<IBig>| x.into_iter().cloned().map(TO::saturating_cast).collect())
}
