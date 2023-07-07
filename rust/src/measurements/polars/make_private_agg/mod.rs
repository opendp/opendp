use crate::combinators::assert_components_match;
use polars::prelude::*;

use crate::core::{Function, Measure, Measurement, Metric, MetricSpace};
use crate::domains::{ExprDomain, LazyGroupByDomain};
use crate::error::*;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    arguments(measurement(rust_type = b"null")), 
    generics(T(suppress))
)]
/// Make a Transformation that returns a Measurement in agg context.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | `input_metric`                             |
/// | ------------------------------- | ------------------------------------------ |
/// | `LazyGroupByDomain`             | `SymmetricDistance`                        |
/// | `LazyGroupByDomain`             | `InsertDeleteDistance`                     |
/// | `LazyGroupByDomain`             | `ChangeOneDistance` if Margins provided    |
/// | `LazyGroupByDomain`             | `HammingDistance` if Margins provided      |
/// | `LazyGroupByDomain`             | `Lp<p, AbsoluteDistance>`                  |
///
/// # Arguments
/// * `input_domain` - LazyGroupByDomain.
/// * `input_metric` - The metric space under which neighboring LazyGroupBy frames are compared.
///
/// # Generics
/// * `T` - AggMeasurement.
///
pub fn make_private_agg<T: AggMeasurement>(
    input_domain: LazyGroupByDomain,
    input_metric: T::InputMetric,
    measurement: Measurement<
        ExprDomain<LazyGroupByDomain>,
        Vec<Expr>,
        T::InputMetric,
        T::OutputMeasure,
    >,
) -> Fallible<Measurement<LazyGroupByDomain, LazyFrame, T::InputMetric, T::OutputMeasure>>
where
    (LazyGroupByDomain, T::InputMetric): MetricSpace,
{
    let function = measurement.function.clone();
    Measurement::new(
        input_domain,
        Function::new_fallible(
            move |frame: &LazyGroupBy| -> Fallible<LazyFrame> {
                let frame = Arc::new(frame.clone());
                let exprs = function.eval(&(frame.clone(), all()))?;
    
                let frame = Arc::try_unwrap(frame)
                    .map_err(|_| err!(FailedFunction, "measurements are not allowed to have side-effects"))?;
                Ok(frame.agg(&exprs))
            },
        ),
        input_metric,
        measurement.output_measure.clone(),
        measurement.privacy_map.clone(),
    )
}

/// Either a `Measurement` or a `PartialMeasurement` that can be used in the select context.
pub trait AggMeasurement: 'static {
    type InputMetric: 'static + Metric;
    type OutputMeasure: 'static + Measure;
    fn fix(
        self,
        input_domain: &LazyGroupByDomain,
        input_metric: &Self::InputMetric,
    ) -> Fallible<Measurement<LazyGroupByDomain, LazyFrame, Self::InputMetric, Self::OutputMeasure>>;
}

impl<MI: 'static + Metric, MO: 'static + Measure> AggMeasurement
    for Measurement<LazyGroupByDomain, LazyFrame, MI, MO>
where
    (LazyGroupByDomain, MI): MetricSpace,
{
    type InputMetric = MI;
    type OutputMeasure = MO;
    fn fix(self, input_domain: &LazyGroupByDomain, input_metric: &MI) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<MI: 'static + Metric, MO: 'static + Measure> AggMeasurement
    for crate::core::PartialMeasurement<LazyGroupByDomain, LazyFrame, MI, MO>
where
    (LazyGroupByDomain, MI): MetricSpace,
{
    type InputMetric = MI;
    type OutputMeasure = MO;
    fn fix(
        self,
        input_domain: &LazyGroupByDomain,
        input_metric: &MI,
    ) -> Fallible<Measurement<LazyGroupByDomain, LazyFrame, MI, MO>> {
        self.fix(input_domain.clone(), input_metric.clone())
    }
}

#[cfg(test)]
mod test_make_agg_meas {
    use polars::prelude::*;

    use crate::combinators::make_basic_composition;
    use crate::core::PartialMeasurement;
    use crate::domains::{AtomDomain, LazyFrameDomain, SeriesDomain, LazyGroupByContext};
    use crate::measurements::polars::make_private_agg;
    use crate::measurements::{make_laplace_expr, then_laplace_expr};
    use crate::metrics::{AbsoluteDistance, Lp};
    use crate::transformations::{make_col, then_col};

    use super::*;

    fn get_test_data() -> Fallible<(
        ExprDomain<LazyGroupByDomain>,
        LazyGroupBy,
        LazyGroupByDomain,
    )> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<f64>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_counts(df!["A"=>[1.0, 2.0], "count" => [1u32, 1]]?.lazy())?;

        let grouping_columns = vec!["A".to_string()];

        let expr_domain = ExprDomain::new(
            frame_domain.clone(),
            LazyGroupByContext {
                columns: grouping_columns.clone(),
            },
            None
        );

        let lazy_gb = df!(
            "A" => &[1.0, 2.0],
            "B" => &[1.0, 1.0],)?
        .lazy()
        .groupby(vec![col("A")]);

        let lazy_gb_domain = LazyGroupByDomain {
            lazy_frame_domain: frame_domain,
            grouping_columns,
        };

        Ok((expr_domain, lazy_gb, lazy_gb_domain))
    }

    #[test]
    fn test_make_agg_trans_output_lazy_frame() -> Fallible<()> {
        let (expr_domain, lazy_gb, lf_gb_domain) = get_test_data()?;

        let col = make_col(
            expr_domain.clone(),
            Lp(AbsoluteDistance::default()),
            "B".to_string(),
        )
        .unwrap_test();

        let scale: f64 = 1.0;
        let laplace = make_laplace_expr(
            col.output_domain.clone(),
            col.output_metric.clone(),
            scale,
        )
        .unwrap_test();

        let meas = (col >> laplace)?;

        let agg_meas = make_private_agg::<Measurement<_, _, Lp<1, AbsoluteDistance<f64>>, _>>(
            lf_gb_domain,
            Lp(AbsoluteDistance::default()),
            make_basic_composition(vec![&meas]).unwrap_test(),
        );

        let _ = agg_meas
            .unwrap_test()
            .invoke(&lazy_gb)
            .unwrap_test()
            .collect()
            .unwrap_test();

        Ok(())
    }

    #[test]
    #[cfg(feature = "partials")]
    fn test_make_agg_trans_partials() -> Fallible<()> {
        let (expr_domain, lazy_gb, lf_gb_domain) = get_test_data()?;
        let space = (lf_gb_domain, Lp(AbsoluteDistance::default()));
        let expr_space = (expr_domain, Lp(AbsoluteDistance::default()));
        let scale: f64 = 1.0;

        let agg_meas = (space
            >> then_private_agg::<PartialMeasurement<_, _, Lp<1, AbsoluteDistance<f64>>, _>>(
                make_basic_composition(vec![&(expr_space
                    >> then_col("B".to_string())
                    >> then_laplace_expr(scale))
                .unwrap_test()])
                .unwrap_test(),
            ))?;
        let _ = agg_meas
            .invoke(&lazy_gb)
            .unwrap_test()
            .collect()
            .unwrap_test();

        Ok(())
    }
}
