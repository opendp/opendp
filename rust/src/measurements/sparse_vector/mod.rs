use std::any::Any;

use crate::{
    core::{Domain, Function, Measure, Measurement, PrivacyMap},
    domains::{AllDomain, QueryableDomain},
    error::Fallible,
    interactive::{Queryable, QueryableBase},
    measures::MaxDivergence,
    metrics::AbsoluteDistance,
    traits::{samplers::SampleDiscreteLaplaceZ2k, CheckNull, Float, InfCast},
};

pub fn make_sparse_vector<DI: 'static + Domain, T: Float, QI, MODE>(
    input_domain: DI,
    threshold: T,
    scale_threshold: T,
    scale_aggregate: T,
    scale_release: MODE::Scale,
    query_limit: usize,
) -> Fallible<
    Measurement<
        QueryableDomain<DI, AllDomain<T>>,
        QueryableDomain<DI, AllDomain<MODE::Output>>,
        AbsoluteDistance<QI>,
        MaxDivergence<T>,
    >,
>
where
    DI::Carrier: 'static + Clone,
    T: 'static + Clone + SampleDiscreteLaplaceZ2k + PartialOrd + InfCast<QI>,
    MaxDivergence<T>: Measure<Distance = T>,
    QI: Clone,
    MODE: SVMode<T>,
{
    Ok(Measurement::new(
        QueryableDomain::new(input_domain.clone(), AllDomain::new()),
        QueryableDomain::new(input_domain.clone(), AllDomain::new()),
        Function::new_fallible(enclose!(scale_release, move |arg: &Queryable<
            DI,
            AllDomain<T>,
        >| {
            let mut trans_queryable = arg.clone();

            let threshold =
                T::sample_discrete_laplace_Z2k(threshold, scale_threshold.clone(), -100)?;
            let scale_aggregate = scale_aggregate.clone();
            let scale_release = scale_release.clone();
            let mut query_limit = query_limit.clone();

            Ok(Queryable::new(
                move |_self: &QueryableBase, query: &dyn Any| {
                    if let Some(query) = query.downcast_ref::<DI::Carrier>() {
                        if query_limit == 0 {
                            return fallible!(FailedFunction, "queries exhausted");
                        }

                        let aggregate = trans_queryable.eval(query)?;
                        let aggregate = T::sample_discrete_laplace_Z2k(
                            aggregate,
                            scale_aggregate.clone(),
                            -100,
                        )?;

                        if aggregate >= threshold {
                            query_limit -= 1;
                        }

                        return Ok(Box::new(MODE::eval_release(
                            aggregate >= threshold,
                            aggregate,
                            scale_release.clone(),
                        )?));
                    }

                    fallible!(FailedFunction, "query not recognized")
                },
            ))
        })),
        AbsoluteDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &QI| {
            let d_in = T::inf_cast(d_in.clone())?;
            let _2 = T::one() + T::one();
            let k_ = T::exact_int_cast(query_limit)?;

            let eps_threshold = d_in.inf_div(&scale_threshold)?;
            let eps_aggregate = _2.inf_mul(&k_)?.inf_mul(&d_in)?.inf_div(&scale_aggregate)?;
            let eps_release = if let Some(scale_release) = MODE::option_scale(scale_release.clone())
            {
                k_.inf_mul(&d_in)?.inf_div(&scale_release)?
            } else {
                T::zero()
            };

            eps_threshold.inf_add(&eps_aggregate)?.inf_add(&eps_release)
        }),
    ))
}

pub trait SVMode<T> {
    type Output: 'static + CheckNull;
    type Scale: 'static + Clone;
    fn eval_release(passes: bool, value: T, scale: Self::Scale) -> Fallible<Self::Output>;
    fn option_scale(scale: Self::Scale) -> Option<T>;
}

struct OnlyBool;
struct WithValue;

impl<T> SVMode<T> for OnlyBool {
    type Output = bool;
    type Scale = ();
    fn eval_release(passes: bool, _value: T, _scale: Self::Scale) -> Fallible<Self::Output> {
        Ok(passes)
    }
    fn option_scale(_scale: Self::Scale) -> Option<T> {
        None
    }
}

impl<T: 'static + Clone + CheckNull> SVMode<T> for WithValue
where
    T: SampleDiscreteLaplaceZ2k,
{
    type Output = Option<T>;
    type Scale = T;
    fn eval_release(passes: bool, value: T, scale: Self::Scale) -> Fallible<Self::Output> {
        passes
            .then(|| T::sample_discrete_laplace_Z2k(value, scale, -100))
            .transpose()
    }
    fn option_scale(scale: Self::Scale) -> Option<T> {
        Some(scale)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sparse_vector() -> Fallible<()> {
        let sv_meas = make_sparse_vector::<AllDomain<f64>, f64, f64, OnlyBool>(
            AllDomain::new(),
            100., // threshold
            4.,   // noise scale for threshold
            6.,   // noise scale for aggregates
            (),   // noise scale for releases
            3,    // limit on number of queries released
        )?;

        let mut sv = sv_meas.invoke(&Queryable::new(|_self: &QueryableBase, query: &dyn Any| {
            Ok(Box::new(*query.downcast_ref::<f64>().unwrap()))
        }))?;

        println!("too small       : {:?}", sv.eval(&1.)?);
        println!("maybe true      : {:?}", sv.eval(&100.)?);
        println!("definitely true : {:?}", sv.eval(&1000.)?);
        println!("maybe exhausted : {:?}", sv.eval(&1000.).is_err());
        println!("exhausted       : {:?}", sv.eval(&1000.).is_err());
        
        Ok(())
    }
}
