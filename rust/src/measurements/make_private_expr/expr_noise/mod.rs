use crate::core::{Measure, Metric, MetricSpace, PrivacyMap};
use crate::measurements::{gaussian_zcdp_map, get_discretization_consts, laplace_puredp_map};
use crate::measures::ZeroConcentratedDivergence;
use crate::metrics::{L1Distance, L2Distance};
use crate::polars::{apply_plugin, match_plugin, ExprFunction, OpenDPPlugin};
use crate::traits::samplers::{
    sample_discrete_gaussian, sample_discrete_gaussian_Z2k, sample_discrete_laplace,
    sample_discrete_laplace_Z2k,
};
use crate::traits::{ExactIntCast, Float, FloatBits, InfCast, InfMul};
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
    measures::MaxDivergence,
    traits::SaturatingCast,
};
use dashu::{integer::IBig, rational::RBig};

use polars::chunked_array::ChunkedArray;
use polars::datatypes::{
    ArrayFromIter, DataType, Field, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type,
    Int8Type, PolarsDataType, UInt32Type, UInt64Type,
};
use polars::error::PolarsResult;
use polars::error::{polars_bail, polars_err};
use polars::lazy::dsl::Expr;
use polars::series::{IntoSeries, Series};
use polars_plan::dsl::{GetOutput, SeriesUdf};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};
use pyo3_polars::derive::polars_expr;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

/// Arguments for the noise expression
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct NoiseArgs {
    /// The distribution to sample from
    pub distribution: Option<Distribution>,

    /// The scale of the noise.
    ///
    /// Scale may be left None, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    pub scale: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Distribution {
    Laplace,
    Gaussian,
}

pub trait NoiseExprMeasure: 'static + Measure<Distance = f64> {
    type Metric: 'static + Metric<Distance = f64>;
    const DISTRIBUTION: Distribution;
    fn map_function(scale: f64) -> impl Fn(&f64) -> Fallible<f64> + 'static + Send + Sync;
}
impl NoiseExprMeasure for MaxDivergence<f64> {
    type Metric = L1Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Laplace;

    fn map_function(scale: f64) -> impl Fn(&f64) -> Fallible<f64> {
        laplace_puredp_map(scale, 0.)
    }
}
impl NoiseExprMeasure for ZeroConcentratedDivergence<f64> {
    type Metric = L2Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Gaussian;
    fn map_function(scale: f64) -> impl Fn(&f64) -> Fallible<f64> {
        gaussian_zcdp_map(scale, 0.)
    }
}

/// Make a measurement that adds noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the noise will be added
/// * `global_scale` - (Re)scale the noise parameter for the noise distribution
pub fn make_expr_noise<MI: 'static + Metric, MO: NoiseExprMeasure>(
    input_domain: ExprDomain,
    input_metric: MI,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>
where
    Expr: StableExpr<MI, MO::Metric>,
    (ExprDomain, MI): MetricSpace,
    (ExprDomain, MO::Metric): MetricSpace,
{
    let Some((input, args)) = match_noise(&expr)? else {
        return fallible!(MakeMeasurement, "Expected noise function");
    };
    let NoiseArgs {
        scale,
        distribution,
    } = args;

    let t_prior = input.clone().make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "Noise mechanism requires either a scale to be set on the expression or a param to be passed to the constructor"
        );
    }

    let scale = scale.unwrap_or(1.);
    let global_scale = global_scale.unwrap_or(1.);
    let scale = scale.inf_mul(&global_scale)?;

    if middle_domain.active_series()?.nullable {
        return fallible!(
            MakeMeasurement,
            "Noise mechanism requires non-nullable input"
        );
    }
    if let Some(distribution) = distribution {
        if MO::DISTRIBUTION != distribution {
            return fallible!(
                MakeMeasurement,
                "expected {:?} distribution, found {:?}",
                MO::DISTRIBUTION,
                distribution
            );
        }
    };

    let m_noise = Measurement::<_, _, MO::Metric, _>::new(
        middle_domain,
        Function::then_expr(move |input_expr| {
            apply_plugin(
                input_expr,
                expr.clone(),
                NoiseArgs {
                    scale: Some(scale),
                    distribution: Some(MO::DISTRIBUTION),
                },
            )
        }),
        middle_metric,
        MO::default(),
        PrivacyMap::new_fallible(MO::map_function(scale)),
    )?;

    t_prior >> m_noise
}

static NOISE_PLUGIN_NAME: &str = "noise";

/// Determine if the given expression is a noise expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the Noise expression and optional scale of noise
pub(crate) fn match_noise(expr: &Expr) -> Fallible<Option<(&Expr, NoiseArgs)>> {
    let Some((input, args)) = match_plugin(expr, NOISE_PLUGIN_NAME)? else {
        return Ok(None);
    };

    let Ok([input]) = <&[_; 1]>::try_from(input.as_slice()) else {
        return fallible!(MakeMeasurement, "Noise expects a single input expression");
    };

    Ok(Some((input, args)))
}

// Code comment, not documentation:
// When using the plugin API from other languages, the NoiseArgs struct is serialized inside a FunctionExpr::FfiPlugin.
// When using the Rust API directly, the NoiseArgs struct is stored inside an AnonymousFunction.

impl OpenDPPlugin for NoiseArgs {
    fn get_options(&self) -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: NOISE_PLUGIN_NAME,
            ..Default::default()
        }
    }
}

// allow the NoiseArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for NoiseArgs {
    // makes it possible to downcast the AnonymousFunction trait object back to NoiseArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        noise_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| noise_type_udf(fields)))
    }
}

/// Implementation of the noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn noise_udf(inputs: &[Series], kwargs: NoiseArgs) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "noise expects a single input expression");
    };
    let Some(scale) = kwargs.scale else {
        polars_bail!(InvalidOperation: "noise scale parameter must be known");
    };

    if scale.is_sign_negative() {
        polars_bail!(InvalidOperation: "noise scale must be non-negative");
    }

    let Some(distribution) = kwargs.distribution else {
        polars_bail!(InvalidOperation: "distribution must be known. Please use `make_private_lazyframe` or explicitly set the noise distribution.");
    };

    // PT stands for Polars Type
    fn noise_impl_integer<PT: 'static + PolarsDataType>(
        series: &Series,
        scale: f64,
        distribution: Distribution,
    ) -> PolarsResult<Series>
    where
        // the physical (rust) dtype must be able to be converted into a big integer (on the input side)
        for<'a> IBig: From<PT::Physical<'a>>,
        // a big integer must be able to be converted into the physical (rust) dtype (on the output side)
        for<'a> PT::Physical<'a>: SaturatingCast<IBig>,
        // must be able to construct the chunked array corresponding to the physical dtype from an iterator
        for<'a> PT::Array:
            ArrayFromIter<PT::Physical<'a>> + ArrayFromIter<Option<PT::Physical<'a>>>,
        // must be able to convert the chunked array to a series
        ChunkedArray<PT>: IntoSeries,
    {
        let Ok(scale) = RBig::try_from(scale) else {
            polars_bail!(InvalidOperation: "scale must be finite")
        };

        Ok(series
            // unpack the series into a chunked array
            .unpack::<PT>()?
            // apply noise to the non-null values
            .try_apply_nonnull_values_generic(|v| {
                let v = IBig::from(v);
                match distribution {
                    Distribution::Laplace => sample_discrete_laplace(scale.clone()),
                    Distribution::Gaussian => sample_discrete_gaussian(scale.clone()),
                }
                .map(|noise| PT::Physical::saturating_cast(v + noise))
            })?
            // convert the resulting chunked array back to a series
            .into_series())
    }

    // PT stands for Polars Type
    fn noise_impl_float<PT: 'static + PolarsDataType>(
        series: &Series,
        scale: f64,
        distribution: Distribution,
    ) -> PolarsResult<Series>
    where
        // the physical (rust) dtype must be a float
        for<'a> PT::Physical<'a>: Float + InfCast<f64>,
        // for calibration of the discretization constant k
        for<'a> i32: ExactIntCast<<PT::Physical<'a> as FloatBits>::Bits>,
        // must be able to construct the chunked array corresponding to the physical dtype from an iterator
        for<'a> PT::Array:
            ArrayFromIter<PT::Physical<'a>> + ArrayFromIter<Option<PT::Physical<'a>>>,
        // must be able to convert the chunked array to a series
        ChunkedArray<PT>: IntoSeries,
    {
        // cast to the physical (rust) dtype (either f32 or f64)
        let scale = PT::Physical::inf_cast(scale)?;
        let k = get_discretization_consts::<PT::Physical<'static>>(None)?.0;
        Ok(series
            // unpack the series into a chunked array
            .unpack::<PT>()?
            // apply noise to the non-null values
            .try_apply_nonnull_values_generic(|v| match distribution {
                Distribution::Laplace => sample_discrete_laplace_Z2k(v, scale, k),
                Distribution::Gaussian => sample_discrete_gaussian_Z2k(v, scale, k),
            })?
            // convert the resulting chunked array back to a series
            .into_series())
    }

    use DataType::*;
    match series.dtype() {
        UInt32 => noise_impl_integer::<UInt32Type>(series, scale, distribution),
        UInt64 => noise_impl_integer::<UInt64Type>(series, scale, distribution),
        Int8 => noise_impl_integer::<Int8Type>(series, scale, distribution),
        Int16 => noise_impl_integer::<Int16Type>(series, scale, distribution),
        Int32 => noise_impl_integer::<Int32Type>(series, scale, distribution),
        Int64 => noise_impl_integer::<Int64Type>(series, scale, distribution),
        Float32 => noise_impl_float::<Float32Type>(series, scale, distribution),
        Float64 => noise_impl_float::<Float64Type>(series, scale, distribution),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type, found {}", dtype),
    }
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn noise_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "noise expects a single input field")
    };
    use DataType::*;
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
    Ok(field.clone())
}

// generate the FFI plugin for the noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func=noise_type_udf)]
fn noise(inputs: &[Series], kwargs: NoiseArgs) -> PolarsResult<Series> {
    noise_udf(inputs, kwargs)
}
