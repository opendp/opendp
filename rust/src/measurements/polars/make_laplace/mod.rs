use crate::core::PrivacyMap;
use crate::domains::DataTypeFrom;
use crate::measurements::make_base_laplace;
use crate::metrics::AbsoluteDistance;
use crate::traits::samplers::CastInternalRational;
use crate::traits::{ExactIntCast, Float};
use crate::{
    core::{Function, Measurement, MetricSpace},
    domains::{AtomDomain, Context, ExprDomain, ExprMetric, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    traits::samplers::SampleDiscreteLaplaceZ2k,
};
use opendp_derive::bootstrap;
use polars::prelude::*;

#[bootstrap(ffi = false)]
/// Polars operator to make the Laplace noise measurement
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == sqrt(2) * standard_deviation.
pub fn make_laplace_expr<
    C: Context,
    MI,
    TA: Float + CastInternalRational + SampleDiscreteLaplaceZ2k + DataTypeFrom + Send + Sync,
>(
    input_domain: ExprDomain<C>,
    input_metric: MI,
    scale: TA,
    k: Option<i32>,
) -> Fallible<Measurement<ExprDomain<C>, Expr, MI, MaxDivergence<TA>>>
where
    (ExprDomain<C>, MI): MetricSpace,
    MI: ExprMetric<C, InnerMetric = AbsoluteDistance<TA>, Distance = TA>,
    i32: ExactIntCast<TA::Bits>,
    TA::Polars: PolarsNumericType<Native = TA>,
    Series: NamedFrom<Vec<TA>, [TA]>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let lap_meas = make_base_laplace::<VectorDomain<AtomDomain<TA>>>(
        Default::default(),
        Default::default(),
        scale.clone(),
        k,
    )?;

    Measurement::new(
        input_domain,
        Function::new_fallible(enclose!(lap_meas, move |(_frame, expr): &(
            C::Value,
            Expr
        )|
              -> Fallible<Expr> {
            Ok(expr.clone().map(
                enclose!(lap_meas, move |s: Series| {
                    let vec: Vec<TA> = s
                        .unpack::<TA::Polars>()?
                        .into_no_null_iter()
                        .collect::<Vec<_>>();
                    let noisy_vec = lap_meas
                        .invoke(&vec)
                        .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;
                    Ok(Some(Series::new(&s.name(), noisy_vec)))
                }),
                GetOutput::same_type(),
            ))
        })),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in| lap_meas.privacy_map.eval(d_in)),
    )
}

#[cfg(test)]
pub mod test_make_laplace_expr {
    use super::*;
    use crate::metrics::{L1Distance, Lp};
    use crate::transformations::polars_test::{get_test_data, get_grouped_test_data};
    use crate::{
        domains::VectorDomain,
        measurements::make_base_laplace,
    };

    #[test]
    fn test_make_laplace_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let scale: f64 = 1.0;

        let meas = make_laplace_expr(expr_domain, AbsoluteDistance::default(), scale, None)?;
        let meas_res = meas.invoke(&(lazy_frame.clone(), col("B")))?;
        let series_exp = lazy_frame
            .select([meas_res])
            .collect()?
            .column("B")?
            .clone();

        let laplace =
            make_base_laplace(VectorDomain::default(), L1Distance::default(), scale, None)?;
        let result = laplace.invoke(&vec![1.0, 2.0, 2.0])?;
        let series_res = Series::new("B", result);

        assert_ne!(series_exp, series_res);
        Ok(())
    }

    #[test]
    fn test_make_laplace_grouped() -> Fallible<()> {
        let (expr_domain, lazy_groupby) = get_grouped_test_data()?;
        let scale: f64 = 1.0;

        let meas = make_laplace_expr(expr_domain, Lp(AbsoluteDistance::default()), scale, None)?;
        let meas_res = meas.invoke(&(lazy_groupby.clone(), col("B")))?;
        let series_res = lazy_groupby.agg([meas_res]).collect()?.column("B")?.clone();

        let chain = make_base_laplace(VectorDomain::default(), L1Distance::default(), scale, None)?;
        let result = chain.invoke(&vec![1.0, 2.0, 2.0])?;
        let series_exp = Series::new("B", result);

        assert_ne!(series_res, series_exp);
        Ok(())
    }
}
