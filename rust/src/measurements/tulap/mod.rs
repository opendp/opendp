use rug::{Integer, Rational};

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AllDomain, SizedDomain, VectorDomain},
    error::Fallible,
    measures::FixedSmoothedMaxDivergence,
    metrics::{InsertDeleteDistance, IntDistance},
    traits::{
        samplers::{sample_discrete_laplace, sample_standard_uniform_rational},
        ExactIntCast, Float,
    },
};

use super::get_discretization_consts;

pub fn make_base_tulap<Q: Float>(
    size: usize,
    eps: Q,
    del: Q,
    k: Option<i32>,
) -> Fallible<
    Measurement<
        SizedDomain<VectorDomain<AllDomain<Rational>>>,
        VectorDomain<AllDomain<Rational>>,
        InsertDeleteDistance,
        FixedSmoothedMaxDivergence<Q>,
    >,
>
where
    i32: ExactIntCast<Q::Bits>,
    Rational: TryFrom<Q>,
{
    let (k, _relaxation): (i32, Q) = get_discretization_consts(k)?;
    let denom = Integer::from(1) >> k;

    let v = (-eps).inf_exp()?;
    let scale =
        Rational::try_from(v).map_err(|_e| err!(MakeMeasurement, "epsilon is too small"))?;

    let del_func =
        Rational::try_from(del).map_err(|_e| err!(MakeMeasurement, "del must be finite"))?;

    let half = Rational::from((1, 2));

    Ok(Measurement::new(
        SizedDomain::new(VectorDomain::new_all(), size),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<Rational>| {
            arg.iter()
                .map(|v: &Rational| {
                    sample_tulap(scale.clone(), denom.clone(), half.clone(), del_func.clone())
                        .map(|sample| v + sample)
                })
                .collect()
        }),
        InsertDeleteDistance::default(),
        FixedSmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &IntDistance| {
            let d_in = Q::inf_cast(*d_in)?;
            // TODO: consider moving this tranformation out into a separate combinator
            Ok((d_in.inf_mul(&eps)?, group_privacy_delta(eps, del, d_in)?))
        }),
    ))
}

fn sample_tulap(
    scale: Rational,
    denom: Integer,
    half: Rational,
    _del: Rational,
) -> Fallible<Rational> {
    let lap = sample_discrete_laplace(scale)?.as_rational().clone();
    let uni = sample_standard_uniform_rational(denom)? - half;
    // TODO: add rejection sampling
    Ok(lap + uni)
}

fn group_privacy_delta<Q: Float>(eps: Q, del: Q, k: Q) -> Fallible<Q> {
    let numer = k.inf_mul(&eps)?.inf_exp()?.inf_sub(&Q::one())?;
    let denom = Q::E().neg_inf_sub(&Q::one())?;
    numer.inf_div(&denom)?.inf_mul(&del)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_tulap_function() -> Fallible<()> {
        let meas = make_base_tulap(10, 1., 1e-8, None)?;
        let sample = meas.invoke(&vec![Rational::new(); 10])?;

        let sample = sample
            .into_iter()
            .map(|v: Rational| v.to_f64())
            .collect::<Vec<_>>();

        println!("{:?}", sample);

        Ok(())
    }
}
