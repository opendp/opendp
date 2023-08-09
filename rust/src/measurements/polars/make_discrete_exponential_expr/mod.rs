use opendp_derive::bootstrap;
use polars::prelude::*;

use crate::{
    core::{Function, Measurement, MetricSpace, PrivacyMap, ExprFunction},
    domains::{ExprDomain, OuterMetric, NumericDataType},
    error::Fallible,
    measurements::{make_base_discrete_exponential, Optimize},
    measures::MaxDivergence,
    metrics::{LInfDiffDistance, L1},
    traits::{DistanceConstant, ExactIntCast, Float, Number}
};

use crate::traits::samplers::CastInternalRational;

#[bootstrap(
    features("contrib"), 
    arguments(optimize(c_type = "char *"))
)]
/// Makes a Measurement to implement the discrete exponential mechanism with Polars.
/// Takes a series of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `temperature` - Higher temperatures are more private.
/// * `optimize` - Indicate whether to privately return the "Max" or "Min"
///
/// # Generics
/// * `MI` - Input Metric.
/// * `QO` - Output Distance Type.
pub fn make_discrete_exponential_expr<MI, QO>(
    input_domain: ExprDomain<MI::LazyDomain>,
    input_metric: MI,
    temperature: QO,
    optimize: Optimize,
) -> Fallible<Measurement<ExprDomain<MI::LazyDomain>, Expr, MI, MaxDivergence<QO>>>
where
    MI: DExpOuterMetric,
    (ExprDomain<MI::LazyDomain>, MI): MetricSpace,

    MI::Distance: Number + CastInternalRational + NumericDataType,
    QO: Float + DistanceConstant<MI::Distance>,
{
    let discrete_exponential = make_base_discrete_exponential::<MI::Distance, QO>(
        Default::default(),
        LInfDiffDistance::default(),
        temperature.clone(),
        optimize.clone(),
    )?;

    let function = discrete_exponential.function.clone();

    Measurement::new(
        input_domain,
        Function::new_expr(move |expr: Expr| -> Expr {
            expr.apply(
                enclose!(function, move |s: Series| {
                    let scores = (s.unpack::<<MI::Distance as NumericDataType>::NumericPolars>()?)
                        .into_no_null_iter()
                        .collect::<Vec<MI::Distance>>();

                    let selected = function.eval(&scores)?;

                    Ok(Some(Series::new(
                        &s.name(),
                        &[u32::exact_int_cast(selected)?],
                    )))
                }),
                GetOutput::from_type(DataType::UInt32),
            )
        }),
        input_metric,
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in| discrete_exponential.map(d_in)),
    )
}

pub trait DExpOuterMetric: OuterMetric {}

impl<T: Number> DExpOuterMetric for LInfDiffDistance<T> {}
impl<T: Number> DExpOuterMetric for L1<LInfDiffDistance<T>> {}



#[cfg(test)]
mod test_discrete_exponential_expr {

    use crate::{
        domains::{AtomDomain, LazyFrameDomain, LazyGroupByContext, SeriesDomain, LazyFrameContext},
        metrics::{LInfDiffDistance, Lp},
    };

    use super::*;

    #[test]
    fn test_make_discrete_exponential_expr_select() -> Fallible<()> {
        let frame_domain =
            LazyFrameDomain::new(vec![SeriesDomain::new("B", AtomDomain::<u64>::default())])?;
        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyFrameContext::Select,
            active_column: Some("B".to_string()),
        };
        let cell: Vec<u64> = vec![22_000, 2_000, 8_000];
        let lazy_frame = DataFrame::new(vec![Series::new("B", &cell)])?.lazy();

        // Get resulting index (expression result)
        let input_metric: LInfDiffDistance<u64> = LInfDiffDistance::default();
        let meas = make_discrete_exponential_expr(expr_domain, input_metric, 1., Optimize::Min)?;
        let expr_meas = meas.invoke(&(Arc::new(lazy_frame.clone()), col("B")))?;

        let release = lazy_frame.select([expr_meas]).collect()?;

        println!("{:?}", release);
        Ok(())
    }

    #[test]
    fn test_make_discrete_exponential_expr_groupby() -> Fallible<()> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<u64>::default()),
        ])?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            active_column: Some("B".to_string()),
        };

        // Output from scoring
        let b_cells: Vec<u64> = vec![20_000, 0, 25_000, 0, 15_000, 15_000];
        let lazy_groupby = DataFrame::new(vec![
            Series::new("A", &[1, 1, 1, 2, 2, 2]),
            Series::new("B", &b_cells),
        ])?
        .lazy()
        .groupby([col("A")]);

        // Get resulting index (expression result)
        let input_metric: LInfDiffDistance<u64> = LInfDiffDistance::default();
        let meas =
            make_discrete_exponential_expr(expr_domain, Lp(input_metric), 1., Optimize::Min)?;
        let expr_meas = meas.invoke(&(Arc::new(lazy_groupby.clone()), col("B")))?;
        let release = lazy_groupby.agg([expr_meas]).collect()?;

        // TODO: why is it packing the selected index into a vec?
        println!("{:?}", release);
        Ok(())
    }
}
