use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    metrics::{AbsoluteDistance, LpDistance},
    traits::Number,
};

pub fn make_vec<T, const P: usize, Q>(
    (input_domain, input_metric): (AtomDomain<T>, AbsoluteDistance<Q>),
) -> Fallible<
    Transformation<
        AtomDomain<T>,
        VectorDomain<AtomDomain<T>>,
        AbsoluteDistance<Q>,
        LpDistance<P, Q>,
    >,
>
where
    T: Number,
    Q: Number,
{
    Transformation::new(
        input_domain.clone(),
        VectorDomain::new(input_domain).with_size(1),
        Function::new(move |arg: &T| vec![arg.clone()]),
        input_metric,
        LpDistance::default(),
        StabilityMap::new(Clone::clone),
    )
}

pub(crate) fn then_index<T: Clone + Default>(i: usize) -> Function<Vec<T>, T> {
    Function::new(move |vec: &Vec<T>| vec.get(i).cloned().unwrap_or_default())
}

// pub fn make_scalarize<
//     MO: 'static + Measure,
//     TIA: Number,
//     TO: 'static + Clone,
//     const P: usize,
//     QI: Number,
// >(
//     measurement: Measurement<VectorDomain<AtomDomain<TIA>>, Vec<TO>, LpDistance<P, QI>, MO>,
// ) -> Fallible<Measurement<AtomDomain<TIA>, TO, AbsoluteDistance<QI>, MO>> {
//     let input_domain = measurement.input_domain.element_domain.clone();
//     let input_metric = AbsoluteDistance::default();
//     make_vec(input_domain, input_metric)?
//         >> measurement
//         >> Function::new(|v: &Vec<TO>| v[0].clone())
// }
