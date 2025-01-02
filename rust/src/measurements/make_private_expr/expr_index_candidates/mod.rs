use crate::core::{Measure, Metric, MetricSpace};
use crate::domains::{ExprPlan, WildExprDomain};
use crate::polars::{apply_plugin, literal_value_of, match_plugin, OpenDPPlugin};
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use polars::datatypes::{DataType, Field};
use polars::error::polars_bail;
#[cfg(feature = "ffi")]
use polars::error::polars_err;
use polars::error::PolarsResult;
use polars::prelude::{Column, CompatLevel};
use polars::series::Series;
use polars_plan::dsl::{ColumnsUdf, Expr, GetOutput};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};
#[cfg(feature = "ffi")]
use pyo3_polars::derive::polars_expr;
#[cfg(feature = "ffi")]
use serde::{Deserialize, Serialize};

use super::PrivateExpr;

#[cfg(test)]
mod test;

/// Make a post-processor that selects from a candidate set.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the Laplace noise will be added
/// * `param` - (Re)scale the noise parameter for the laplace distribution
pub fn make_expr_index_candidates<MI: 'static + Metric, MO: 'static + Measure>(
    input_domain: WildExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    param: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (WildExprDomain, MI): MetricSpace,
{
    let (input, IndexCandidatesPlugin { candidates }) =
        match_index_candidates(&expr)?.ok_or_else(|| {
            err!(
                MakeMeasurement,
                "Expected {:?} function",
                IndexCandidatesShim::NAME
            )
        })?;

    let m_prior = input
        .clone()
        .make_private(input_domain, input_metric, output_measure, param)?;

    m_prior
        >> Function::then_expr(move |input_expr| {
            apply_plugin(
                input_expr,
                expr.clone(),
                IndexCandidatesPlugin {
                    candidates: candidates.clone(),
                },
            )
        })
}

/// Determine if the given expression is an index_candidates expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the Laplace expression and the scale of the Laplace noise
pub(crate) fn match_index_candidates(
    expr: &Expr,
) -> Fallible<Option<(&Expr, IndexCandidatesPlugin)>> {
    let Some(input) = match_plugin::<IndexCandidatesShim>(expr)? else {
        return Ok(None);
    };

    let Ok([input, candidates]) = <&[_; 2]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "{:?} expects two inputs: data and candidates",
            IndexCandidatesShim::NAME
        );
    };

    let candidates = literal_value_of::<Series>(candidates)?
        .ok_or_else(|| err!(MakeTransformation, "candidates must be known"))?;

    Ok(Some((input, IndexCandidatesPlugin { candidates })))
}

#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub(crate) struct IndexCandidatesShim;
impl ColumnsUdf for IndexCandidatesShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Deserialize, Serialize))]
pub(crate) struct IndexCandidatesPlugin {
    pub candidates: Series,
}

impl OpenDPPlugin for IndexCandidatesShim {
    const NAME: &'static str = "index_candidates";
    fn function_options() -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: Self::NAME,
            ..Default::default()
        }
    }

    fn get_output(&self) -> Option<GetOutput> {
        // dtype is unknown
        Some(GetOutput::from_type(DataType::Null))
    }
}

impl OpenDPPlugin for IndexCandidatesPlugin {
    const NAME: &'static str = "index_candidates_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: Self::NAME,
            ..Default::default()
        }
    }

    fn get_output(&self) -> Option<GetOutput> {
        let dtype = self.candidates.0.dtype().clone();
        Some(GetOutput::map_field(move |f| {
            Ok(Field::new(f.name().clone(), dtype.clone()))
        }))
    }
}

// allow the IndexCandidatesArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for IndexCandidatesPlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Option<Column>> {
        index_candidates_udf(s, self.clone()).map(Some)
    }
}

/// Implementation of the index_candidates noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn index_candidates_udf(inputs: &[Column], kwargs: IndexCandidatesPlugin) -> PolarsResult<Column> {
    let Ok([column]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{:?} expects a single input field", IndexCandidatesShim::NAME);
    };
    let selections = kwargs.candidates.0.take(column.u32()?)?;
    Ok(Column::Series(selections.with_name(column.name().clone())))
}

// generate the FFI plugin for the index_candidates noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type=Null)]
fn index_candidates(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn index_candidates_plugin_type_udf(
    input_fields: &[Field],
    kwargs: IndexCandidatesPlugin,
) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "{:?} expects a single input field", IndexCandidatesShim::NAME)
    };

    if field.dtype() != &DataType::UInt32 {
        polars_bail!(InvalidOperation: "Expected u32 input field, found {:?}", field.dtype())
    }

    Ok(Field::new(
        field.name().clone(),
        kwargs.candidates.0.dtype().clone(),
    ))
}

// generate the FFI plugin for the index_candidates noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func_with_kwargs=index_candidates_plugin_type_udf)]
fn index_candidates_plugin(
    inputs: &[Series],
    kwargs: IndexCandidatesPlugin,
) -> PolarsResult<Series> {
    let inputs: Vec<Column> = inputs.iter().cloned().map(Column::Series).collect();
    let out = index_candidates_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
