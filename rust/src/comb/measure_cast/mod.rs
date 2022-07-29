use crate::{
    core::{Domain, Measurement, Metric, PrivacyMap},
    error::Fallible,
    measures::{SMDCurve, SmoothedMaxDivergence, ZeroConcentratedDivergence},
    traits::Float,
};

#[cfg(feature="ffi")]
mod ffi;

pub fn make_cast_zcdp_approxdp<DI, DO, MI, QO>(
    meas: Measurement<DI, DO, MI, ZeroConcentratedDivergence<QO>>,
) -> Fallible<Measurement<DI, DO, MI, SmoothedMaxDivergence<QO>>>
where
    DI: Domain,
    DO: Domain,
    MI: 'static + Metric,
    QO: Float,
{
    let Measurement {
        input_domain,
        output_domain,
        function,
        input_metric,
        privacy_map,
        ..
    } = meas;

    Ok(Measurement::new(
        input_domain,
        output_domain,
        function,
        input_metric,
        SmoothedMaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let _2 = QO::one() + QO::one();
            let rho = privacy_map.eval(d_in)?;
            if rho.is_sign_negative() {
                return fallible!(FailedRelation, "rho must be non-negative");
            }
            Ok(SMDCurve::new(move |&delta: &QO| {
                if delta.is_sign_negative() {
                    return fallible!(FailedRelation, "delta must be non-negative");
                }

                if delta >= QO::one() || rho.is_zero() {
                    return Ok(QO::zero());
                }

                // maintain cdp_delta(rho,eps) >= delta
                let mut epsmin = QO::zero();

                // to compute epsmax we use the standard bound
                // maintain cdp_delta(rho,eps) <= delta
                let mut epsmax = delta
                    .recip()
                    .inf_ln()?
                    .inf_mul(&rho)?
                    .inf_sqrt()?
                    .inf_mul(&_2)?
                    .inf_add(&rho)?;

                for _ in 0..1000 {
                    let eps = (epsmin + epsmax) / _2;
                    if cdp_delta(rho, eps)? <= delta {
                        epsmax = eps;
                    } else {
                        epsmin = eps;
                    }
                }
                Ok(epsmax)
            }))
        }),
    ))
}

fn cdp_delta<Q>(rho: Q, eps: Q) -> Fallible<Q>
where
    Q: Float,
{
    // assert rho>=0
    // assert eps>=0

    if rho.is_sign_negative() {
        return fallible!(FailedRelation, "rho must be non-negative");
    }

    if rho.is_zero() {
        return Ok(Q::zero());
    }

    //search for best alpha
    //Note that any alpha in (1,infty) yields a valid upper bound on delta
    // Thus if this search is slightly "incorrect" it will only result in larger delta (still valid)
    // This code has two "hacks".
    // First the binary search is run for a pre-specificed length.
    // 1000 iterations should be sufficient to converge to a good solution.
    // Second we set a minimum value of alpha to avoid numerical stability issues.
    // Note that the optimal alpha is at least (1+eps/rho)/2. Thus we only hit this constraint
    // when eps<=rho or close to it. This is not an interesting parameter regime, as you will
    // inherently get large delta in this regime.

    // don't let alpha be too small, due to numerical stability
    let _1 = Q::one();
    let _2 = _1 + _1;
    let mut amin = Q::round_cast(1.01f64)?;
    let mut amax = (eps + _1) / (_2 * rho) + _2;
    let mut alpha = (amin + amax) / _2;

    //should be enough iterations
    for _ in 0..1000 {
        let derivative = (_2 * alpha - _1) * rho - eps + alpha.recip().neg().inf_ln_1p()?;
        if derivative.is_sign_negative() {
            amin = alpha;
        } else {
            amax = alpha;
        }
        alpha = (amin + amax) / _2;
    }

    // now calculate delta
    let delta = ((alpha - _1) * (alpha * rho - eps) + alpha * alpha.recip().neg().inf_ln_1p()?)
        .inf_exp()?
        / (alpha - _1);
    // delta<=1 always
    Ok(delta.min(Q::one()))
}
