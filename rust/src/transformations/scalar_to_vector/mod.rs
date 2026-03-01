use opendp_derive::proven;

use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, LpDistance},
    traits::CheckAtom,
};

#[proven(proof_path = "transformations/scalar_to_vector/make_vec.tex")]
pub fn make_vec<T, const P: usize, Q>(
    (input_domain, input_metric): (AtomDomain<T>, AbsoluteDistance<Q>),
) -> Fallible<
    Transformation<
        AtomDomain<T>,
        AbsoluteDistance<Q>,
        VectorDomain<AtomDomain<T>>,
        LpDistance<P, Q>,
    >,
>
where
    T: CheckAtom,
    Q: 'static + Clone,
{
    Transformation::new(
        input_domain.clone(),
        input_metric,
        VectorDomain::new(input_domain).with_size(1),
        LpDistance::default(),
        Function::new(move |arg: &T| vec![arg.clone()]),
        StabilityMap::new(Clone::clone),
    )
}

#[proven(proof_path = "transformations/scalar_to_vector/then_index_or_default.tex")]
pub(crate) fn then_index_or_default<T: Clone + Default>(i: usize) -> Function<Vec<T>, T> {
    Function::new(move |vec: &Vec<T>| vec.get(i).cloned().unwrap_or_default())
}
