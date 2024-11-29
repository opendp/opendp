use crate::core::PrivacyMap;
use crate::domains::{AtomDomain, ExprPlan, VectorDomain, WildExprDomain};
use crate::measurements::{Optimize, SelectionMeasure, make_report_noisy_max};
use crate::measures::MaxDivergence;
use crate::metrics::{IntDistance, L0InfDistance, L01InfDistance, LInfDistance};
use crate::polars::{OpenDPPlugin, apply_plugin, literal_value_of, match_plugin};
use crate::traits::{InfCast, InfMul, Number};
use crate::transformations::StableExpr;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};
use dashu::float::FBig;

use polars::datatypes::{
    DataType, Field, Float32Type, Float64Type, Int8Type, Int16Type, Int32Type, Int64Type,
    PolarsDataType, UInt32Type, UInt64Type,
};
use polars::error::polars_bail;
#[cfg(feature = "ffi")]
use polars::error::polars_err;
use polars::error::{PolarsError, PolarsResult};
use polars::lazy::dsl::Expr;
use polars::prelude::{Column, CompatLevel, IntoColumn};
use polars::series::{IntoSeries, Series};
use polars_arrow::array::PrimitiveArray;
use polars_arrow::types::NativeType;
use polars_plan::dsl::{ColumnsUdf, GetOutput};
use polars_plan::prelude::FunctionOptions;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};

use super::approximate_c_stability;

#[cfg(test)]
mod test;

/// Make a measurement that reports the index of the maximum value after adding noise.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the selection will be applied
/// * `global_scale` - (Re)scale the noise distribution
pub fn make_expr_report_noisy_max<MI: 'static + UnboundedMetric, MO: 'static + SelectionMeasure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, L01InfDistance<MI>, MO>>
where
    Expr: StableExpr<L01InfDistance<MI>, L0InfDistance<LInfDistance<f64>>>,
{
    let (input, optimize, scale) = match_report_noisy_max(&expr)?
        .ok_or_else(|| err!(MakeMeasurement, "Expected {}", ReportNoisyMaxPlugin::NAME))?;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "{} requires a scale parameter",
            ReportNoisyMaxPlugin::NAME
        );
    }

    let scale = match scale {
        Some(scale) => scale,
        None => {
            let (l_0, l_inf) = approximate_c_stability(&t_prior)?;
            f64::inf_cast(l_0)?.inf_mul(&l_inf)?
        }
    };
    let global_scale = global_scale.unwrap_or(1.);

    if scale.is_nan() || scale.is_sign_negative() {
        return fallible!(
            MakeMeasurement,
            "{} scale must be a non-negative number",
            ReportNoisyMaxPlugin::NAME
        );
    }
    if global_scale.is_nan() || global_scale.is_sign_negative() {
        return fallible!(
            MakeMeasurement,
            "global_scale ({}) must be a non-negative number",
            global_scale
        );
    }

    let scale = scale.inf_mul(&global_scale)?;

    if middle_domain.column.nullable {
        return fallible!(
            MakeMeasurement,
            "{} requires non-nullable input",
            ReportNoisyMaxPlugin::NAME
        );
    }

    let DataType::Array(elem_dtype, _) = middle_domain.column.dtype() else {
        return fallible!(
            MakeMeasurement,
            "{} requires array input",
            ReportNoisyMaxPlugin::NAME
        );
    };

    use DataType::*;
    if !matches!(
        elem_dtype.as_ref(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        return fallible!(
            MakeMeasurement,
            "{} requires numeric array input",
            ReportNoisyMaxPlugin::NAME
        );
    }
    let m_rnm = make_report_noisy_max(
        VectorDomain::<AtomDomain<f64>>::default(),
        middle_metric.0.clone(),
        MO::default(),
        scale,
        optimize,
    )?;
    let privacy_map = m_rnm.privacy_map.clone();

    t_prior
        >> Measurement::<_, _, L0InfDistance<LInfDistance<f64>>, _>::new(
            middle_domain,
            Function::then_expr(move |input_expr| {
                apply_plugin(
                    vec![input_expr],
                    expr.clone(),
                    ReportNoisyMaxPlugin {
                        optimize: optimize.clone(),
                        scale,
                    },
                )
            }),
            middle_metric.clone(),
            MO::default(),
            PrivacyMap::new_fallible(move |(l0, li): &(IntDistance, f64)| {
                let epsilon = privacy_map.eval(li)?;
                f64::inf_cast(*l0)?.inf_mul(&epsilon)
            }),
        )?
}

/// Determine if the given expression is a report_noisy_max_gumbel expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the report_noisy_max_gumbel expression and the arguments to the mechanism
pub(crate) fn match_report_noisy_max(
    expr: &Expr,
) -> Fallible<Option<(&Expr, Optimize, Option<f64>)>> {
    let Some(input) = match_plugin::<ReportNoisyMaxShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, optimize, scale]) = <&[_; 3]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "{:?} expects three inputs",
            ReportNoisyMaxShim::NAME
        );
    };

    let optimize = literal_value_of::<String>(optimize)?.ok_or_else(|| {
        err!(
            MakeMeasurement,
            "Optimize must be \"max\" or \"min\", found \"{}\".",
            optimize
        )
    })?;
    let optimize = Optimize::deserialize(optimize.as_str().into_deserializer())
        .map_err(|e: serde::de::value::Error| err!(FailedFunction, "{:?}", e))?;

    let scale = literal_value_of::<f64>(scale)?;

    Ok(Some((data, optimize, scale)))
}

/// Arguments for the Report Noisy Max Gumbel noise expression
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ReportNoisyMaxShim;
impl ColumnsUdf for ReportNoisyMaxShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Column]) -> PolarsResult<Option<Column>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }
}

impl OpenDPPlugin for ReportNoisyMaxShim {
    const NAME: &'static str = "report_noisy_max";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise().with_fmt_str(Self::NAME)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            report_noisy_max_plugin_type_udf(fields)
        }))
    }
}

/// Arguments for the Report Noisy Max expression
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ReportNoisyMaxPlugin {
    /// Controls whether the noisy maximum or noisy minimum candidate is selected.
    pub optimize: Optimize,
    /// The scale of the noise.
    pub scale: f64,
}

impl OpenDPPlugin for ReportNoisyMaxPlugin {
    const NAME: &'static str = "report_noisy_max_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions::elementwise().with_fmt_str(Self::NAME)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            report_noisy_max_plugin_type_udf(fields)
        }))
    }
}

// allow the ReportNoisyMaxPlugin struct to be stored inside an AnonymousFunction, when used from Rust directly
impl ColumnsUdf for ReportNoisyMaxPlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Column]) -> PolarsResult<Option<Column>> {
        report_noisy_max_gumbel_udf(s, self.clone()).map(Some)
    }
}

/// Implementation of the report_noisy_max_gumbel noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn report_noisy_max_gumbel_udf(
    inputs: &[Column],
    kwargs: ReportNoisyMaxPlugin,
) -> PolarsResult<Column> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", ReportNoisyMaxPlugin::NAME);
    };

    let ReportNoisyMaxPlugin { optimize, scale } = kwargs;

    if scale.is_sign_negative() {
        polars_bail!(InvalidOperation: "{} scale ({}) must be non-negative", ReportNoisyMaxPlugin::NAME, scale);
    }

    if scale.is_nan() {
        polars_bail!(InvalidOperation: "{} scale ({}) must be a number", ReportNoisyMaxPlugin::NAME, scale);
    }

    // PT stands for Polars Type
    fn rnm_impl<PT: 'static + PolarsDataType>(
        column: &Column,
        scale: f64,
        optimize: Optimize,
    ) -> PolarsResult<Column>
    where
        // the physical (rust) dtype must be a number that can be converted into a rational
        PT::Physical<'static>: NativeType + Number,
        FBig: TryFrom<PT::Physical<'static>>,
        f64: InfCast<PT::Physical<'static>>,
    {
        let m_rnm = make_report_noisy_max(
            VectorDomain::<AtomDomain<PT::Physical<'static>>>::default(),
            LInfDistance::default(),
            MaxDivergence,
            scale,
            optimize,
        )?;
        Ok(column
            .as_materialized_series()
            // unpack the series into a chunked array
            .array()?
            // apply RNM max to each value
            .try_apply_nonnull_values_generic::<UInt32Type, _, _, _>(move |v| {
                let arr = v
                    .as_any()
                    .downcast_ref::<PrimitiveArray<PT::Physical<'static>>>()
                    .ok_or_else(|| {
                        PolarsError::InvalidOperation("input dtype does not match".into())
                    })?;

                let idx = m_rnm.invoke(&arr.values_iter().cloned().collect())? as u32;
                PolarsResult::Ok(idx)
            })?
            // convert the resulting chunked array back to a series
            .into_series()
            .into_column())
    }

    use DataType::*;
    let Array(dtype, _) = series.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", series.dtype())
    };

    match dtype.as_ref() {
        UInt32 => rnm_impl::<UInt32Type>(series, scale, optimize),
        UInt64 => rnm_impl::<UInt64Type>(series, scale, optimize),
        Int8 => rnm_impl::<Int8Type>(series, scale, optimize),
        Int16 => rnm_impl::<Int16Type>(series, scale, optimize),
        Int32 => rnm_impl::<Int32Type>(series, scale, optimize),
        Int64 => rnm_impl::<Int64Type>(series, scale, optimize),
        Float32 => rnm_impl::<Float32Type>(series, scale, optimize),
        Float64 => rnm_impl::<Float64Type>(series, scale, optimize),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type found {}", dtype),
    }
}

#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type=Null)]
fn report_noisy_max(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn report_noisy_max_plugin_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "{} expects a single input field", ReportNoisyMaxPlugin::NAME)
    };
    use DataType::*;
    let Array(dtype, n) = field.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", field.dtype())
    };

    if *n == 0 {
        polars_bail!(InvalidOperation: "Array must have a non-zero length");
    }

    if matches!(dtype.as_ref(), UInt8 | UInt16) {
        polars_bail!(
            InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64."
        );
    }
    if !matches!(
        dtype.as_ref(),
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64 | Float32 | Float64
    ) {
        polars_bail!(
            InvalidOperation: "Expected numeric data type, found {:?}",
            field.dtype()
        );
    }
    Ok(Field::new(field.name().clone(), UInt32))
}

// generate the FFI plugin for the report_noisy_max_gumbel expression
#[cfg(feature = "ffi")]
#[pyo3_polars::derive::polars_expr(output_type_func=report_noisy_max_plugin_type_udf)]
fn report_noisy_max_plugin(
    inputs: &[Series],
    kwargs: ReportNoisyMaxPlugin,
) -> PolarsResult<Series> {
    let inputs: Vec<Column> = inputs.iter().cloned().map(|s| s.into_column()).collect();
    let out = report_noisy_max_gumbel_udf(inputs.as_slice(), kwargs)?;
    Ok(out.take_materialized_series())
}
