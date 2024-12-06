use std::{collections::HashMap, sync::Arc};

use opendp_derive::bootstrap;
use opendp_polars_sql::{function_registry::FunctionRegistry, SQLContext};
use polars::{
    error::PolarsResult,
    frame::DataFrame,
    prelude::{Column, DslPlan, IntoLazy, LazySerde, PlSmallStr, SpecialEq},
    series::Series,
};
use polars_plan::dsl::UserDefinedFunction;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::{DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    measurements::{
        expr_index_candidates::IndexCandidatesShim, expr_noise::NoiseShim,
        expr_report_noisy_max::ReportNoisyMaxShim, make_private_lazyframe,
    },
    polars::{OnceFrame, OpenDPPlugin},
    transformations::expr_discrete_quantile_score::DiscreteQuantileScoreShim,
};

use super::PrivateDslPlan;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null"),
        threshold(rust_type = "Option<u32>", c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress))
)]
/// Create a differentially private measurement from a SQL query.
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `output_measure` - How to measure privacy loss.
/// * `sql` - The sql query
/// * `global_scale` - Optional. A tune-able parameter that affects the privacy-utility tradeoff.
/// * `threshold` - Optional. Minimum number of rows in each released partition.
pub fn make_private_sql<MI: Metric, MO: 'static + Measure>(
    input_domain: LazyFrameDomain,
    input_metric: MI,
    output_measure: MO,
    query: &str,
    global_scale: Option<f64>,
    threshold: Option<u32>,
) -> Fallible<Measurement<LazyFrameDomain, OnceFrame, MI, MO>>
where
    DslPlan: PrivateDslPlan<MI, MO>,
    (DslPlanDomain, MI): MetricSpace,
    (LazyFrameDomain, MI): MetricSpace,
{
    macro_rules! register {
        ($($ident:ident),+) => ([$(
            (<$ident>::NAME.into(), UserDefinedFunction {
                name: <$ident>::NAME.into(),
                input_fields: Vec::new(),
                return_type: $ident.get_output(),
                fun: LazySerde::Deserialized(SpecialEq::new(Arc::new($ident))),
                options: <$ident>::function_options(),
            })
        ),+])
    }

    let registry = ODPFunctionRegistry {
        registry: HashMap::from(register![
            IndexCandidatesShim,
            NoiseShim,
            ReportNoisyMaxShim,
            DiscreteQuantileScoreShim
        ]),
    };

    let mut context = SQLContext::new().with_function_registry(Arc::new(registry));

    let columns = input_domain
        .schema()
        .into_iter()
        .map(|(name, dtype)| {
            let series = Series::from_any_values_and_dtype(name, &[], &dtype, false)?;
            Ok(Column::Series(series))
        })
        .collect::<Fallible<_>>()?;
    context.register("data", DataFrame::new(columns)?.lazy());

    make_private_lazyframe(
        input_domain,
        input_metric,
        output_measure,
        context.execute(query)?,
        global_scale,
        threshold,
    )
}

#[derive(Default)]
struct ODPFunctionRegistry {
    registry: HashMap<PlSmallStr, UserDefinedFunction>,
}

impl FunctionRegistry for ODPFunctionRegistry {
    fn register(&mut self, name: &str, fun: UserDefinedFunction) -> PolarsResult<()> {
        self.registry.insert(name.into(), fun);
        Ok(())
    }

    fn get_udf(&self, name: &str) -> PolarsResult<Option<UserDefinedFunction>> {
        Ok(self.registry.get(name).cloned())
    }

    fn contains(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }
}
