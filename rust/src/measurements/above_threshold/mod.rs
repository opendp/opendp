use std::cmp::Ordering;

use rug::{float::Round, ops::AssignRound, Float as RFloat};

use crate::{
    core::{Function, Measurement, MetricSpace, PrivacyMap},
    domains::AllDomain,
    error::Fallible,
    interactive::Queryable,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{
        samplers::{LaplacePSRN, PSRN},
        Float, InfCast, Number,
    },
};

macro_rules! float {
    ($val:expr, $round:ident) => {
        RFloat::with_val_round(f64::MANTISSA_DIGITS, $val, Round::$round).0
    };
}

pub fn make_above_threshold<TI, QI: Number, QO: Float>(
    input_domain: AllDomain<Queryable<TI, QI>>,
    input_metric: LInfDistance<QI>,
    scale: QO,
    threshold: QI,
) -> Fallible<
    Measurement<
        AllDomain<Queryable<TI, QI>>,
        Queryable<TI, bool>,
        LInfDistance<QI>,
        MaxDivergence<QO>,
    >,
>
where
    TI: 'static,
    QO: InfCast<QI>,
    (AllDomain<Queryable<TI, QI>>, LInfDistance<QI>): MetricSpace,

    RFloat: AssignRound<QI, Round = Round, Ordering = Ordering>,
    RFloat: AssignRound<QO, Round = Round, Ordering = Ordering>,
{
    let _2 = QO::exact_int_cast(2)?;
    let threshold = float!(threshold, Up);

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Queryable<TI, QI>| {
            let scale = float!(scale.inf_mul(&_2)?, Up);

            let mut trans_queryable = arg.clone();
            let mut found = false;
            // TODO: twice as much noise as necessary if monotonic
            // TODO: can be reused in other instances of above_threshold
            let mut threshold_psrn = LaplacePSRN::new(threshold.clone(), scale.clone());
            let scale = scale.clone();

            Queryable::new_external(move |query: &TI| {
                if found {
                    return fallible!(FailedFunction, "queries exhausted");
                }

                let aggregate = float!(trans_queryable.eval(query)?, Down);
                let mut aggregate_psrn = LaplacePSRN::new(aggregate, scale.clone());

                Ok(if aggregate_psrn.greater_than(&mut threshold_psrn)? {
                    found = true;
                    true
                } else {
                    false
                })
            })
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &QI| {
            let d_in = input_metric.range_distance(*d_in)?;

            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }

            if d_in.is_zero() {
                return Ok(QO::zero());
            }

            if scale.is_zero() {
                return Ok(QO::infinity());
            }

            // d_in / scale
            d_in.inf_div(&scale)
        }),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sparse_vector() -> Fallible<()> {
        let sv_meas = make_above_threshold::<f64, f64, f64>(
            AllDomain::default(),
            LInfDistance::monotonic(),
            100., // threshold
            4.,   // noise scale for threshold
        )?;

        let mut sv = sv_meas.invoke(&Queryable::new_external(|query: &f64| Ok(*query))?)?;

        println!("too small       : {:?}", sv.eval(&1.)?);
        println!("maybe true      : {:?}", sv.eval(&100.).is_err());
        println!("definitely true : {:?}", sv.eval(&1000.).is_err());
        println!("exhausted       : {:?}", sv.eval(&1000.).is_err());

        Ok(())
    }
}
