use crate::core::PrivacyMap;
use crate::domains::DataTypeFrom;
use crate::measurements::{get_discretization_consts, make_base_laplace};
use crate::metrics::AbsoluteDistance;
use crate::traits::samplers::CastInternalRational;
use crate::traits::{ExactIntCast, Float, Number};
use crate::transformations::ToVec;
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
    QI: Number,
>(
    input_domain: ExprDomain<C>,
    input_metric: MI,
    scale: TA,
    k: Option<i32>,
) -> Fallible<Measurement<ExprDomain<C>, Expr, MI, MaxDivergence<TA>>>
where
    (ExprDomain<C>, MI): MetricSpace,
    MI: ExprMetric<C, InnerMetric = AbsoluteDistance<QI>, Distance = TA>,
    ChunkedArray<TA::Polars>: ToVec<TA>,
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

    let (k, _): (i32, TA) = get_discretization_consts(k)?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |(_frame, expr): &(C::Value, Expr)| -> Fallible<Expr> {
            Ok(expr.clone().map(
                move |s: Series| {
                    let noisy_vec: Vec<TA> = s
                        .unpack::<TA::Polars>()?
                        .into_no_null_iter()
                        .map(|value| TA::sample_discrete_laplace_Z2k(scale.clone(), value, k))
                        .collect::<Fallible<Vec<_>>>()
                        .map_err(|e| PolarsError::ComputeError(e.to_string().into()))?;

                    // TODO: use this when threading supported
                    // let noisy_vec = lap_meas.invoke(&vec).unwrap();
                    Ok(Some(Series::new(&s.name(), noisy_vec)))
                },
                GetOutput::same_type(),
            ))
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in| lap_meas.privacy_map.eval(d_in)),
    )
}

#[cfg(test)]
mod test_make_laplace_expr {
    use super::*;
    use crate::metrics::{L1Distance, Lp};
    use crate::{
        domains::{
            AtomDomain, LazyFrameContext, LazyFrameDomain, LazyGroupByContext, SeriesDomain,
            VectorDomain,
        },
        measurements::make_base_laplace,
    };

    fn get_test_data() -> Fallible<(ExprDomain<LazyFrameContext>, LazyFrame)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_counts(df!["count" => [1u32]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyFrameContext::Select,
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2],
            "B" => &[1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame))
    }

    fn get_grouped_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [1u32, 1]]?.lazy())?;

        let lazy_frame = df!(
            "A" => &[1, 2],
            "B" => &[1.0, 2.0],)?
        .lazy();

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            active_column: Some("B".to_string()),
        };

        Ok((expr_domain, lazy_frame.groupby_stable([col("A")])))
    }

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
        let result = laplace.invoke(&vec![1.0, 2.0])?;
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
        let result = chain.invoke(&vec![1.0, 2.0])?;
        let series_exp = Series::new("B", result);

        assert_ne!(series_res, series_exp);
        Ok(())
    }
}
