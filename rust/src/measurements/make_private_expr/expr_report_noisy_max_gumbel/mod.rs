use crate::core::{
    apply_plugin, match_plugin, ExprFunction, Metric, MetricSpace, OpenDPPlugin, PrivacyMap, Scalar,
};
use crate::measurements::{rnm_gumbel_map, select_score, Optimize};
use crate::metrics::{IntDistance, LInfDistance, Parallel};
use crate::traits::samplers::CastInternalRational;
use crate::traits::{InfCast, InfMul, Number};
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
    measures::MaxDivergence,
};
use dashu::rational::RBig;

use polars::datatypes::{
    DataType, Field, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
    PolarsDataType, UInt32Type, UInt64Type,
};
use polars::error::{polars_bail, polars_err};
use polars::error::{PolarsError, PolarsResult};
use polars::lazy::dsl::Expr;
use polars::series::{IntoSeries, Series};
use polars_arrow::array::PrimitiveArray;
use polars_arrow::types::NativeType;
use polars_plan::dsl::{GetOutput, SeriesUdf};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};
use pyo3_polars::derive::polars_expr;
use serde::{Deserialize, Serialize};

/// Make a measurement that reports the index of the maximum value after adding Gumbel noise.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the selection will be applied
/// * `param` - (Re)scale the noise parameter for the Gumbel distribution
pub fn make_expr_report_noisy_max_gumbel<MI: 'static + Metric>(
    input_domain: ExprDomain,
    input_metric: MI,
    expr: Expr,
    param: f64,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MaxDivergence<f64>>>
where
    Expr: StableExpr<MI, Parallel<LInfDistance<Scalar>>>,
    (ExprDomain, MI): MetricSpace,
{
    let (input, RNMGumbelArgs { scale, optimize }) = match_rnm_gumbel(&expr)?
        .ok_or_else(|| err!(MakeMeasurement, "Expected RNM Gumbel function"))?;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_nan() && param.is_nan() {
        return fallible!(
            MakeMeasurement,
            "RNM Gumbel mechanism requires a scale parameter"
        );
    }

    let scale = if scale.is_nan() { 1. } else { scale };
    let param = if param.is_nan() { 1. } else { param };
    let param = param.inf_mul(&scale)?;

    if middle_domain.active_series()?.nullable {
        return fallible!(
            MakeMeasurement,
            "RNM Gumbel mechanism requires non-nullable input"
        );
    }
    let monotonic = middle_metric.0.monotonic;

    t_prior
        >> Measurement::<_, _, Parallel<LInfDistance<Scalar>>, _>::new(
            middle_domain,
            Function::new_expr(move |input_expr| {
                apply_plugin(
                    input_expr,
                    expr.clone(),
                    RNMGumbelArgs {
                        scale: param,
                        optimize: optimize.clone(),
                    },
                )
            }),
            middle_metric,
            MaxDivergence::default(),
            PrivacyMap::new_fallible(move |(l0, li): &(IntDistance, Scalar)| {
                let f64_metric = LInfDistance::new(monotonic);
                let epsilon = rnm_gumbel_map(param, f64_metric)(&li.f64()?)?;
                f64::inf_cast(*l0)?.inf_mul(&epsilon)
            }),
        )?
}

/// Determine if the given expression is a RNM Gumbel expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the RNM Gumbel expression and the arguments to the mechanism
pub(crate) fn match_rnm_gumbel(expr: &Expr) -> Fallible<Option<(&Expr, RNMGumbelArgs)>> {
    let Some((input, args)) = match_plugin(expr, "rnm_gumbel")? else {
        return Ok(None);
    };

    let Ok([input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(
            MakeMeasurement,
            "RNM Gumbel expects a single input expression"
        );
    };

    Ok(Some((input, args)))
}

/// Arguments for the Report Noisy Max Gumbel noise expression
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct RNMGumbelArgs {
    /// The scale of the Gumbel noise.
    ///
    /// When parsed by [`make_private_expr`], NaN defaults to one and is multiplied by the `param`.
    pub scale: f64,
    /// Controls whether the noisy maximum or noisy minimum candidate is selected.
    pub optimize: String,
}

impl OpenDPPlugin for RNMGumbelArgs {
    fn get_options(&self) -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: "rnm_gumbel",
            ..Default::default()
        }
    }
}

// allow the RNMGumbelArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for RNMGumbelArgs {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        rnm_gumbel_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            rnm_gumbel_type_udf(fields)
                // NOTE: it would be better if this didn't need to fall back,
                // but Polars does not support raising an error
                .ok()
                .unwrap_or_else(|| fields[0].clone())
        }))
    }
}

/// Implementation of the RNM Gumbel noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn rnm_gumbel_udf(inputs: &[Series], kwargs: RNMGumbelArgs) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "RNM Gumbel expects a single input field");
    };

    let scale = RBig::try_from(kwargs.scale)
        .map_err(|_| PolarsError::InvalidOperation("scale must be finite".into()))?;
    let optimize = Optimize::try_from(kwargs.optimize.as_ref())?;

    // PT stands for Polars Type
    fn rnm_gumbel_impl<PT: 'static + PolarsDataType>(
        series: &Series,
        scale: RBig,
        optimize: Optimize,
    ) -> PolarsResult<Series>
    where
        // the physical (rust) dtype must be a number that can be converted into a rational
        for<'a> PT::Physical<'a>: NativeType + Number + CastInternalRational,
    {
        Ok(series
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

                select_score(arr.values_iter().cloned(), optimize.clone(), scale.clone())
                    .map(|idx| idx as u32)
            })?
            // convert the resulting chunked array back to a series
            .into_series())
    }

    use DataType::*;
    let Array(dtype, _) = series.dtype() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", series.dtype())
    };

    match dtype.as_ref() {
        UInt32 => rnm_gumbel_impl::<UInt32Type>(series, scale, optimize),
        UInt64 => rnm_gumbel_impl::<UInt64Type>(series, scale, optimize),
        Int8 => rnm_gumbel_impl::<Int8Type>(series, scale, optimize),
        Int16 => rnm_gumbel_impl::<Int16Type>(series, scale, optimize),
        Int32 => rnm_gumbel_impl::<Int32Type>(series, scale, optimize),
        Int64 => rnm_gumbel_impl::<Int64Type>(series, scale, optimize),
        Float32 => rnm_gumbel_impl::<Float32Type>(series, scale, optimize),
        Float64 => rnm_gumbel_impl::<Float64Type>(series, scale, optimize),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type found {}", dtype),
    }
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn rnm_gumbel_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "rnm_gumbel expects a single input field")
    };
    use DataType::*;
    let Array(dtype, n) = field.data_type() else {
        polars_bail!(InvalidOperation: "Expected array data type, found {:?}", field.data_type())
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
        UInt32 | UInt64 | Int8 | Int16 | Int32 | Int64
    ) {
        polars_bail!(
            InvalidOperation: "Expected integer data type, found {:?}",
            field.data_type()
        );
    }
    Ok(Field::new(field.name(), UInt32))
}

// generate the FFI plugin for the RNM Gumbel noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func=rnm_gumbel_type_udf)]
fn rnm_gumbel(inputs: &[Series], kwargs: RNMGumbelArgs) -> PolarsResult<Series> {
    rnm_gumbel_udf(inputs, kwargs)
}

#[cfg(test)]
mod test_expr_rnm_gumbel {
    use super::*;
    use polars::prelude::*;

    use crate::{
        core::PrivacyNamespaceHelper,
        measurements::make_private_expr,
        metrics::{PartitionDistance, SymmetricDistance},
        transformations::expr_discrete_quantile_score::test_expr_quantile::get_quantile_test_data,
    };

    #[test]
    fn test_rnm_gumbel_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_quantile_test_data()?;
        let expr_domain = lf_domain.select();
        let scale: f64 = 1e-8;
        let candidates = vec![0., 10., 20., 30., 40., 50., 60., 70., 80., 90., 100.];

        let m_quant = make_private_expr(
            expr_domain,
            PartitionDistance(SymmetricDistance),
            MaxDivergence::default(),
            col("A").dp().quantile(candidates, 0.5, scale),
            scale,
        )?;

        let dp_expr = m_quant.invoke(&(lf.logical_plan.clone(), all()))?;
        let df = lf.select([dp_expr]).collect()?;
        let actual = df.column("A")?.u32()?.get(0).unwrap();
        assert_eq!(actual, 5);

        Ok(())
    }
}
