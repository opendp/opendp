use polars::{
    datatypes::{
        ArrayChunked, ArrowDataType,
        DataType::{self, *},
        Field, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, StaticArray,
        UInt32Type, UInt64Type,
    },
    error::{PolarsResult, polars_bail, polars_err},
    prelude::{Column, CompatLevel, IntoColumn, PolarsPhysicalType},
    series::Series,
};
#[cfg(feature = "ffi")]
use polars_arrow as arrow;

use polars_arrow::{
    array::{FixedSizeListArray, UInt64Array},
    datatypes::Field as ArrowField,
};
use polars_plan::{
    dsl::{ColumnsUdf, GetOutput},
    prelude::FunctionOptions,
};

#[cfg(feature = "ffi")]
use pyo3_polars::derive::polars_expr;

use serde::{Deserialize, Serialize};

use crate::{polars::OpenDPPlugin, traits::RoundCast, transformations::score_candidates};

use super::series_to_vec;

#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
pub(crate) struct DiscreteQuantileScoreShim;
impl ColumnsUdf for DiscreteQuantileScoreShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for DiscreteQuantileScoreShim {
    const NAME: &'static str = "discrete_quantile_score";
    const SHIM: bool = true;
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }

    fn get_output(&self) -> Option<GetOutput> {
        // dtype is unknown
        Some(GetOutput::from_type(DataType::Array(Box::new(UInt64), 1)))
    }
}

/// Arguments for the discrete quantile score expression
#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
pub(crate) struct DiscreteQuantileScorePlugin {
    /// Candidates to score
    pub candidates: Series,
    /// Alpha numerator, alpha denominator
    pub alpha: (u64, u64),
    // Max group length
    pub size_limit: u64,
}

impl OpenDPPlugin for DiscreteQuantileScorePlugin {
    const NAME: &'static str = "discrete_quantile_score_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::aggregation()
    }

    fn get_output(&self) -> Option<GetOutput> {
        let kwargs = self.clone();
        Some(GetOutput::map_fields(move |fields| {
            discrete_quantile_score_plugin_type_udf(fields, kwargs.clone())
        }))
    }
}

// allow the DiscreteQuantileScoreArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for DiscreteQuantileScorePlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to DiscreteQuantileScoreArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Option<Column>> {
        discrete_quantile_score_udf(s, self.clone()).map(Some)
    }
}

/// Implementation of the discrete quantile score expression.
///
/// The Polars engine executes this function over chunks of data.
fn discrete_quantile_score_udf(
    inputs: &[Column],
    kwargs: DiscreteQuantileScorePlugin,
) -> PolarsResult<Column> {
    let Ok([column]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", DiscreteQuantileScoreShim::NAME);
    };
    let series = column.as_materialized_series();

    // pack scores into a series, where each row is an array of the scores for each candidate
    let scores = match series.dtype() {
        UInt32 => compute_scores_array::<UInt32Type>(series, kwargs),
        UInt64 => compute_scores_array::<UInt64Type>(series, kwargs),
        Int8 => compute_scores_array::<Int8Type>(series, kwargs),
        Int16 => compute_scores_array::<Int16Type>(series, kwargs),
        Int32 => compute_scores_array::<Int32Type>(series, kwargs),
        Int64 => compute_scores_array::<Int64Type>(series, kwargs),
        Float32 => compute_scores_array::<Float32Type>(series, kwargs),
        Float64 => compute_scores_array::<Float64Type>(series, kwargs),
        UInt8 | UInt16 => polars_bail!(
            InvalidOperation: "u8 and u16 are not supported in the OpenDP Polars plugin. Please use u32 or u64."),
        dtype => polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            dtype),
    }?;

    let dtype = ArrowDataType::FixedSizeList(
        // should match how Polars initializes ArrowField
        Box::new(ArrowField::new("item".into(), ArrowDataType::UInt64, true)),
        // the length here is the number of elements in each row
        scores.len(),
    );

    let fsla = FixedSizeListArray::new(dtype, 1, Box::new(scores), None);
    Ok(Series::from(ArrayChunked::from(fsla)).into_column())
}

// PT stands for Polars Type
/// Glue for calling the scorer function from Polars/Arrow dtypes.
fn compute_scores_array<PT: 'static + PolarsPhysicalType>(
    series: &Series,
    kwargs: DiscreteQuantileScorePlugin,
) -> PolarsResult<UInt64Array>
where
    // candidates must be able to be converted into a physical dtype
    for<'a> PT::Physical<'a>: 'static + RoundCast<f64> + PartialOrd,
    PT::Array: StaticArray,
{
    let DiscreteQuantileScorePlugin {
        candidates,
        alpha: (alpha_num, alpha_den),
        size_limit,
    } = kwargs;

    Ok(UInt64Array::from_values(score_candidates(
        series
            .unpack::<PT>()?
            .downcast_iter()
            .flat_map(StaticArray::values_iter),
        series_to_vec::<PT>(&candidates.cast(&PT::get_static_dtype())?)?,
        alpha_num,
        alpha_den,
        size_limit,
    )))
}

#[cfg(feature = "ffi")]
#[polars_expr(output_type=Null)]
fn discrete_quantile_score(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn discrete_quantile_score_plugin_type_udf(
    input_fields: &[Field],
    kwargs: DiscreteQuantileScorePlugin,
) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "DQ Score expects a single input field")
    };
    if matches!(field.dtype(), UInt8 | UInt16) {
        polars_bail!(
            InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
        );
    }
    if !matches!(
        field.dtype(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            field.dtype()
        );
    }

    let out_dtype = Array(Box::new(UInt64), kwargs.candidates.0.len());
    Ok(Field::new(field.name().clone(), out_dtype))
}

// generate the FFI plugin for the DQ score expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func_with_kwargs=discrete_quantile_score_plugin_type_udf)]
fn discrete_quantile_score_plugin(
    inputs: &[Series],
    kwargs: DiscreteQuantileScorePlugin,
) -> PolarsResult<Series> {
    let inputs: Vec<Column> = inputs.iter().cloned().map(|s| s.into_column()).collect();
    let out = discrete_quantile_score_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
