use crate::core::{
    apply_plugin, match_plugin, ExprFunction, Measure, Metric, MetricSpace, OpenDPPlugin,
};
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use polars::datatypes::{AnyValue, DataType, Field};
use polars::error::PolarsResult;
use polars::error::{polars_bail, polars_err};
use polars::prelude::NamedFrom;
use polars::series::Series;
use polars_plan::dsl::{Expr, GetOutput, SeriesUdf};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};
use pyo3_polars::derive::polars_expr;
#[cfg(feature = "ffi")]
use serde::{ser::SerializeSeq, Deserialize, Serialize};
#[cfg(feature = "ffi")]
use serde_pickle::Value;

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
    input_domain: ExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    param: Option<f64>,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (ExprDomain, MI): MetricSpace,
{
    let (input, IndexCandidatesArgs { candidates }) = match_index_candidates(&expr)?
        .ok_or_else(|| err!(MakeMeasurement, "Expected Index Candidates function"))?;

    let m_prior = input
        .clone()
        .make_private(input_domain, input_metric, output_measure, param)?;

    m_prior
        >> Function::then_expr(move |input_expr| {
            apply_plugin(
                input_expr,
                expr.clone(),
                IndexCandidatesArgs {
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
) -> Fallible<Option<(&Expr, IndexCandidatesArgs)>> {
    let Some((input, args)) = match_plugin(expr, "index_candidates")? else {
        return Ok(None);
    };

    let Ok([input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "index_candidates expects a single input expression"
        );
    };

    Ok(Some((input, args)))
}

#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Deserialize, Serialize))]
pub(crate) struct IndexCandidatesArgs {
    pub candidates: Candidates,
}

#[derive(Clone)]
pub(crate) struct Candidates(pub Series);

#[cfg(feature = "ffi")]
impl Serialize for Candidates {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        for value in self.0.iter() {
            match value {
                AnyValue::Boolean(v) => seq.serialize_element(&v),
                AnyValue::Int64(v) => seq.serialize_element(&v),
                AnyValue::Float64(v) => seq.serialize_element(&v),
                AnyValue::String(v) => seq.serialize_element(&v),
                _ => Err(serde::ser::Error::custom(
                    "Expected homogenous candidates of either bool, i64, f64, or string",
                )),
            }?;
        }
        seq.end()
    }
}

#[cfg(feature = "ffi")]
impl<'de> Deserialize<'de> for Candidates {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let Value::List(list) = value else {
            return Err(serde::de::Error::custom(format!(
                "Expected a list, found {:?}",
                value
            )));
        };
        let first = list
            .first()
            .ok_or_else(|| serde::de::Error::custom("Expected at least one candidate"))?
            .clone();

        macro_rules! match_candidates {
            ($list:ident, $dtype:ident) => {
                Series::new(
                    "",
                    $list
                        .into_iter()
                        .map(|x| match x {
                            Value::$dtype(v) => Ok(v),
                            _ => Err(serde::de::Error::custom("Expected homogenous candidates")),
                        })
                        .collect::<Result<Vec<_>, D::Error>>()?,
                )
            };
        }

        let candidates = match first {
            Value::Bool(_) => match_candidates!(list, Bool),
            Value::I64(_) => match_candidates!(list, I64),
            Value::F64(_) => match_candidates!(list, F64),
            Value::String(_) => match_candidates!(list, String),
            first => {
                return Err(serde::de::Error::custom(format!(
                    "Candidates must be homogeneous primitives, first candidate is {:?}",
                    first
                )))
            }
        };

        Ok(Candidates(candidates))
    }
}

impl OpenDPPlugin for IndexCandidatesArgs {
    fn get_options(&self) -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: "index_candidates",
            ..Default::default()
        }
    }
}

// allow the IndexCandidatesArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for IndexCandidatesArgs {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        index_candidates_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        let dtype = self.candidates.0.dtype().clone();
        Some(GetOutput::map_field(move |f| {
            Field::new(f.name(), dtype.clone())
        }))
    }
}

/// Implementation of the index_candidates noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn index_candidates_udf(inputs: &[Series], kwargs: IndexCandidatesArgs) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "index_candidates expects a single input field");
    };
    let selections = kwargs.candidates.0.take(series.u32()?)?;
    Ok(selections.with_name(series.name()))
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn index_candidates_type_udf(
    input_fields: &[Field],
    kwargs: IndexCandidatesArgs,
) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "index_candidates expects a single input field")
    };

    if field.data_type() != &DataType::UInt32 {
        polars_bail!(InvalidOperation: "Expected u32 input field, found {:?}", field.data_type())
    }

    Ok(Field::new(
        field.name(),
        kwargs.candidates.0.dtype().clone(),
    ))
}

// generate the FFI plugin for the index_candidates noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func_with_kwargs=index_candidates_type_udf)]
fn index_candidates(inputs: &[Series], kwargs: IndexCandidatesArgs) -> PolarsResult<Series> {
    index_candidates_udf(inputs, kwargs)
}
