use std::iter::zip;

use polars::{
    datatypes::{
        ArrayChunked, ArrowDataType, DataType::*, Field, Float32Type, Float64Type, Int16Type,
        Int32Type, Int64Type, Int8Type, PolarsDataType, StaticArray, UInt32Type, UInt64Type,
    },
    error::{polars_bail, polars_err, PolarsResult},
    series::Series,
};
use polars_arrow::{
    array::{FixedSizeListArray, UInt64Array},
    datatypes::Field as ArrowField,
};
use polars_plan::{
    dsl::{GetOutput, SeriesUdf},
    prelude::{ApplyOptions, FunctionOptions},
};

#[cfg(feature = "ffi")]
use pyo3_polars::derive::polars_expr;
#[cfg(feature = "ffi")]
use serde::{Deserialize, Serialize};

use crate::{
    core::OpenDPPlugin, measurements::expr_index_candidates::Candidates, traits::RoundCast,
};

use super::{series_to_vec, DQ_SCORE_PLUGIN_NAME};

/// Arguments for the discrete quantile score expression
#[derive(Clone)]
#[cfg_attr(feature = "ffi", derive(Serialize, Deserialize))]
pub(crate) struct DiscreteQuantileScoreArgs {
    /// Candidates to score
    pub candidates: Candidates,
    /// A value between [0, 1]
    pub alpha: f64,
    /// Alpha numerator, alpha denominator, and max partition length
    pub constants: Option<(u64, u64, u64)>,
}

impl OpenDPPlugin for DiscreteQuantileScoreArgs {
    fn get_options(&self) -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::GroupWise,
            fmt_str: DQ_SCORE_PLUGIN_NAME,
            returns_scalar: true,
            changes_length: true,
            ..Default::default()
        }
    }
}

// allow the DiscreteQuantileScoreArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for DiscreteQuantileScoreArgs {
    // makes it possible to downcast the AnonymousFunction trait object back to DiscreteQuantileScoreArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        discrete_quantile_score_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        let kwargs = self.clone();
        Some(GetOutput::map_fields(move |fields| {
            discrete_quantile_score_type_udf(fields, kwargs.clone())
                .ok()
                .unwrap_or_else(|| fields[0].clone())
        }))
    }
}

/// Implementation of the discrete quantile score expression.
///
/// The Polars engine executes this function over chunks of data.
fn discrete_quantile_score_udf(
    inputs: &[Series],
    kwargs: DiscreteQuantileScoreArgs,
) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", DQ_SCORE_PLUGIN_NAME);
    };

    let n = series.len() as u64;
    let (Candidates(candidates), constants) = (kwargs.candidates, kwargs.constants);

    // when a user initially creates a DQ score expression, the constants are not yet known and are left empty
    // the constants are computed in make_expr_discrete_quantile_score and embedded in the resulting expression
    let Some((alpha_num, alpha_den, size_limit)) = constants else {
        polars_bail!(InvalidOperation:
            "encountered quantile in expression that has not been made private or stable",
        );
    };
    // compute histograms of the number of records between each candidate
    // one histogram has left-open intervals, the other has right-open intervals
    let (hist_lo, hist_ro) = match series.dtype() {
        UInt32 => series_histogram::<UInt32Type>(series, candidates),
        UInt64 => series_histogram::<UInt64Type>(series, candidates),
        Int8 => series_histogram::<Int8Type>(series, candidates),
        Int16 => series_histogram::<Int16Type>(series, candidates),
        Int32 => series_histogram::<Int32Type>(series, candidates),
        Int64 => series_histogram::<Int64Type>(series, candidates),
        Float32 => series_histogram::<Float32Type>(series, candidates),
        Float64 => series_histogram::<Float64Type>(series, candidates),
        UInt8 | UInt16 => polars_bail!(
            InvalidOperation: "u8 and u16 are not supported in the OpenDP Polars plugin. Please use u32 or u64."),
        dtype => polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            dtype),
    }?;

    let scores_iter = zip(hist_lo, hist_ro)
        .scan((0, 0), |(lt, le), (lo, ro)| {
            // cumsum the left-open histogram to get the total number of records less than the candidate
            *lt += lo;
            // cumsum the right-open histogram to get the total number of records lt or equal to the candidate
            *le += ro;

            let gt = n - *le;

            // the number of records equal to the candidate is the difference between the two cumsums
            Some(((*lt).min(size_limit), gt.min(size_limit)))
        })
        .map(|(lt, gt)| {
            // |(1 - α) * #(x < c)          -       α * #(x > c)  |    * α_den
            ((alpha_den - alpha_num) * lt).abs_diff(alpha_num * gt)
        });

    // pack scores into a series, where each row is an array of the scores for each candidate
    let scores = UInt64Array::from_values(scores_iter);

    let dtype = ArrowDataType::FixedSizeList(
        Box::new(ArrowField::new("", ArrowDataType::UInt64, false)),
        scores.len(),
    );

    let fsla = FixedSizeListArray::new(dtype, Box::new(scores), None);
    Ok(Series::from(ArrayChunked::from(fsla)))
}

// PT stands for Polars Type
fn series_histogram<PT: 'static + PolarsDataType>(
    series: &Series,
    candidates: Series,
) -> PolarsResult<(Vec<u64>, Vec<u64>)>
where
    // candidates must be able to be converted into a the physical dtype
    for<'a> PT::Physical<'a>: 'static + RoundCast<f64> + PartialOrd,
    PT::Array: StaticArray,
{
    let candidates = series_to_vec::<PT>(&candidates.cast(&PT::get_dtype())?)?;

    // count of the number of records between...
    //  (-inf, c1), [c1, c2), [c2, c3), ..., [ck, inf)
    let mut hist_lo = vec![0u64; candidates.len() + 1];
    //  (-inf, c1], (c1, c2], (c2, c3], ..., (ck, inf)
    let mut hist_ro = vec![0u64; candidates.len() + 1];

    series
        .unpack::<PT>()?
        .downcast_iter()
        .flat_map(StaticArray::values_iter)
        .for_each(|v| {
            let idx_lt = candidates.partition_point(|c| *c < v);
            hist_ro[idx_lt] += 1;

            let idx_eq = idx_lt + candidates[idx_lt..].partition_point(|c| *c == v);
            hist_lo[idx_eq] += 1;
        });

    // don't care about the number of elements greater than all
    hist_lo.pop();
    hist_ro.pop();

    Ok((hist_lo, hist_ro))
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn discrete_quantile_score_type_udf(
    input_fields: &[Field],
    kwargs: DiscreteQuantileScoreArgs,
) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "DQ Score expects a single input field")
    };
    if matches!(field.data_type(), UInt8 | UInt16) {
        polars_bail!(
            InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
        );
    }
    if !matches!(
        field.data_type(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            field.data_type()
        );
    }

    let out_dtype = Array(Box::new(UInt64), kwargs.candidates.0.len());
    Ok(Field::new(field.name(), out_dtype))
}

// generate the FFI plugin for the DQ score expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func_with_kwargs=discrete_quantile_score_type_udf)]
fn discrete_quantile_score(
    inputs: &[Series],
    kwargs: DiscreteQuantileScoreArgs,
) -> PolarsResult<Series> {
    discrete_quantile_score_udf(inputs, kwargs)
}
