use std::sync::Arc;

use crate::core::{Measure, Metric, MetricSpace};
use crate::domains::{ExprPlan, WildExprDomain};
use crate::polars::{OpenDPPlugin, apply_plugin, literal_value_of, match_plugin};
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

#[cfg(feature = "ffi")]
use polars::datatypes::DataType;
use polars::datatypes::Field;
use polars::error::polars_bail;
use polars::error::{PolarsResult, polars_err};
use polars::prelude::{AnonymousColumnsUdf, Column, IntoColumn};
use polars::series::Series;
use polars_plan::dsl::{ColumnsUdf, Expr};
use polars_plan::prelude::{FunctionFlags, FunctionOptions};
#[cfg(feature = "ffi")]
use pyo3_polars::derive::polars_expr;
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
) -> Fallible<Measurement<WildExprDomain, MI, MO, ExprPlan>>
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
                vec![input_expr],
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

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Column> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl AnonymousColumnsUdf for IndexCandidatesShim {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }

    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }

    fn get_field(
        &self,
        _: &polars::prelude::Schema,
        fields: &[polars::prelude::Field],
    ) -> PolarsResult<polars::prelude::Field> {
        let [index, cands] = <&[_; 2]>::try_from(fields)
            .map_err(|_| polars_err!(InvalidOperation: "expected two input arguments"))?;
        Ok(Field::new(index.name.clone(), cands.dtype.clone()))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct IndexCandidatesPlugin {
    pub candidates: Series,
}

impl OpenDPPlugin for IndexCandidatesShim {
    const NAME: &'static str = "index_candidates";
    #[cfg(feature = "ffi")]
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        let mut flags = FunctionFlags::default();
        flags.set_elementwise();
        FunctionOptions {
            flags,
            ..Default::default()
        }
    }
}

impl AnonymousColumnsUdf for IndexCandidatesPlugin {
    fn as_column_udf(self: Arc<Self>) -> Arc<dyn ColumnsUdf> {
        self
    }

    fn deep_clone(self: Arc<Self>) -> Arc<dyn AnonymousColumnsUdf> {
        Arc::new(Arc::unwrap_or_clone(self))
    }

    fn get_field(
        &self,
        _: &polars::prelude::Schema,
        fields: &[polars::prelude::Field],
    ) -> PolarsResult<polars::prelude::Field> {
        let dtype = self.candidates.0.dtype().clone();
        let [index] = <&[_; 1]>::try_from(fields)
            .map_err(|_| polars_err!(InvalidOperation: "expected one input argument"))?;
        Ok(Field::new(index.name.clone(), dtype))
    }
}

impl OpenDPPlugin for IndexCandidatesPlugin {
    const NAME: &'static str = "index_candidates_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise()
    }
}

// allow the IndexCandidatesArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for IndexCandidatesPlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Column> {
        index_candidates_udf(s, self.clone())
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
    Ok(selections.with_name(column.name().clone()).into_column())
}

// generate the FFI plugin for the index_candidates noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type=Null)]
fn index_candidates(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

#[cfg(feature = "ffi")]
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
    let inputs: Vec<Column> = inputs.iter().cloned().map(|s| s.into_column()).collect();
    let out = index_candidates_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
