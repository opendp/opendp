use crate::combinators::assert_components_match;
use polars::prelude::*;

use crate::core::{Function, Measure, Measurement, Metric, MetricSpace};
use crate::domains::{ExprDomain, LazyGroupByContext, LazyGroupByDomain};
use crate::error::*;
use opendp_derive::bootstrap;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    arguments(measurement(c_type = "AnyMeasurement *", rust_type = b"null")),
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
pub fn make_private_agg<T: AggMeasurement>(
    input_domain: LazyGroupByDomain,
    input_metric: T::InputMetric,
    measurement: T,
) -> Fallible<Measurement<LazyGroupByDomain, LazyFrame, T::InputMetric, T::OutputMeasure>>
where
    (LazyGroupByDomain, T::InputMetric): MetricSpace,
{
    let expr_domain = ExprDomain {
        lazy_frame_domain: input_domain.lazy_frame_domain.clone(),
        context: LazyGroupByContext {
            columns: input_domain.grouping_columns.clone(),
        },
        active_column: None,
    };
    let measurement = measurement.fix(&expr_domain, &input_metric)?;
    let function = measurement.function.clone();
    Measurement::new(
        input_domain,
        Function::new_fallible(move |frame: &LazyGroupBy| -> Fallible<LazyFrame> {
            let frame = Arc::new(frame.clone());
            let exprs = function.eval(&(frame.clone(), all()))?;

            let frame = Arc::try_unwrap(frame)
                .map_err(|_| err!(FailedFunction, "measurements must not have side-effects"))?;
            Ok(frame.agg(&exprs))
        }),
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
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &Self::InputMetric,
    ) -> Fallible<
        Measurement<
            ExprDomain<LazyGroupByDomain>,
            Vec<Expr>,
            Self::InputMetric,
            Self::OutputMeasure,
        >,
    >;
}

impl<MI: 'static + Metric, MO: 'static + Measure> AggMeasurement
    for Measurement<ExprDomain<LazyGroupByDomain>, Vec<Expr>, MI, MO>
where
    (ExprDomain<LazyGroupByDomain>, MI): MetricSpace,
{
    type InputMetric = MI;
    type OutputMeasure = MO;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &MI,
    ) -> Fallible<Self> {
        assert_components_match!(DomainMismatch, &self.input_domain, input_domain);
        assert_components_match!(MetricMismatch, &self.input_metric, input_metric);

        Ok(self)
    }
}

#[cfg(feature = "partials")]
impl<MI: 'static + Metric, MO: 'static + Measure> AggMeasurement
    for crate::core::PartialMeasurement<ExprDomain<LazyGroupByDomain>, Vec<Expr>, MI, MO>
where
    (ExprDomain<LazyGroupByDomain>, MI): MetricSpace,
{
    type InputMetric = MI;
    type OutputMeasure = MO;
    fn fix(
        self,
        input_domain: &ExprDomain<LazyGroupByDomain>,
        input_metric: &MI,
    ) -> Fallible<Measurement<ExprDomain<LazyGroupByDomain>, Vec<Expr>, MI, MO>> {
        self.fix(input_domain.clone(), input_metric.clone())
    }
}

#[cfg(test)]
mod test_make_agg_meas {
    use polars::prelude::*;

    use crate::combinators::make_basic_composition;
    use crate::domains::{AtomDomain, LazyFrameDomain, LazyGroupByContext, SeriesDomain};
    use crate::measurements::make_laplace_expr;
    use crate::measurements::polars::make_private_agg;
    use crate::metrics::L1Distance;
    use crate::transformations::make_col;

    use super::*;

    fn get_private_agg_test_data() -> Fallible<(LazyGroupByDomain, LazyGroupBy)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<f64>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_counts(df!["A" => [1.0, 2.0], "count" => [1u32, 1]]?.lazy())?;

        let lazy_gb_domain = LazyGroupByDomain {
            lazy_frame_domain: frame_domain,
            grouping_columns: vec!["A".to_string()],
        };

        let lazy_gb = df!(
            "A" => &[1.0, 2.0],
            "B" => &[1.0, 1.0]
        )?
        .lazy()
        .group_by(&[col("A")]);

        Ok((lazy_gb_domain, lazy_gb))
    }

    #[test]
    fn test_private_agg() -> Fallible<()> {
        let (lf_gb_domain, lazy_gb) = get_private_agg_test_data()?;

        let expr_domain = ExprDomain::new(
            lf_gb_domain.lazy_frame_domain.clone(),
            LazyGroupByContext {
                columns: lf_gb_domain.grouping_columns.clone(),
            },
            None,
        );

        let col = make_col(expr_domain.clone(), L1Distance::default(), "B".to_string())?;
        let laplace = make_laplace_expr(col.output_domain.clone(), col.output_metric.clone(), 1.0)?;

        let agg_meas = make_private_agg(
            lf_gb_domain,
            L1Distance::<f64>::default(),
            make_basic_composition(vec![(col >> laplace)?])?,
        )?;

        agg_meas.invoke(&lazy_gb)?.collect()?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "partials")]
    fn test_private_agg_partials() -> Fallible<()> {
        use crate::{
            combinators::then_basic_composition, measurements::then_laplace_expr,
            metrics::L1Distance, transformations::then_col,
        };

        let (lf_gb_domain, lazy_gb) = get_private_agg_test_data()?;

        let agg_meas = (lf_gb_domain, L1Distance::<f64>::default())
            >> then_private_agg(then_basic_composition(vec![
                then_col("B".to_string()) >> then_laplace_expr(1.0f64),
            ]));
        agg_meas?.invoke(&lazy_gb)?.collect()?;

        Ok(())
    }
}
