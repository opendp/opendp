use dashu::{integer::IBig, rational::RBig};

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::LpDistance,
    traits::{CastInternalRational, ExactIntCast, Float, Number},
};

use super::{find_nearest_multiple_of_2k, get_rounding_distance, x_div_2k, x_mul_2k};

pub fn make_integerize_vec<TIA, const P: usize, QI>(
    input_space: (VectorDomain<AtomDomain<TIA>>, LpDistance<P, QI>),
    k: i32,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<IBig>>,
        LpDistance<P, QI>,
        LpDistance<P, RBig>,
    >,
>
where
    TIA: Float,
    QI: Number,
    i32: ExactIntCast<TIA::Bits>,
    RBig: TryFrom<TIA> + TryFrom<QI>,
{
    let (input_domain, input_metric) = input_space;

    let rounding_distance = get_rounding_distance::<TIA>(k, input_domain.size)?;

    let output_domain = VectorDomain {
        element_domain: AtomDomain::<IBig>::default(),
        size: input_domain.size,
    };

    Transformation::new(
        input_domain,
        output_domain,
        Function::new(move |arg: &Vec<TIA>| {
            arg.into_iter()
                .cloned()
                .map(|x_i| {
                    let x_i = RBig::try_from(x_i).unwrap_or(RBig::ZERO);
                    find_nearest_multiple_of_2k(x_i, k)
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

pub fn then_deintegerize_vec<TO: CastInternalRational>(k: i32) -> Function<Vec<IBig>, Vec<TO>> {
    Function::new(move |x: &Vec<IBig>| {
        x.into_iter()
            .cloned()
            .map(|x_i| TO::from_rational(x_mul_2k(x_i, k)))
            .collect()
    })
}
