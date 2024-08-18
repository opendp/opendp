use crate::core::{Measure, Metric, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, NumericDataType, VectorDomain};
use crate::measurements::{make_gaussian, make_laplace, make_noise, GaussianDomain, LaplaceDomain, ZExpFamily};
use crate::measures::ZeroConcentratedDivergence;
use crate::metrics::{L1Distance, L2Distance, PartitionDistance};
use crate::polars::{apply_plugin, literal_value_of, match_plugin, ExprFunction, OpenDPPlugin};
use crate::traits::samplers::{
    sample_discrete_gaussian, sample_discrete_gaussian_Z2k, sample_discrete_laplace,
    sample_discrete_laplace_Z2k,
};
use crate::traits::{CastInternalRational, ExactIntCast, Float, FloatBits, InfCast, InfMul, Number};
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::{get_min_k, StableExpr};
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
    measures::MaxDivergence,
    traits::SaturatingCast,
};
use dashu::{integer::IBig, rational::RBig};
use polars::prelude::PolarsNumericType;
use polars_arrow::array::PrimitiveArray;
use polars_core::utils::Container;
use serde::de::value::Error;

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
use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};

use super::approximate_c_stability;

#[cfg(test)]
mod test;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct NoiseShim;
impl SeriesUdf for NoiseShim {
    // makes it possible to downcast the AnonymousFunction trait object back to Self
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, _: &mut [Series]) -> PolarsResult<Option<Series>> {
        polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noise_plugin_type_udf(fields)
        }))
    }
}

impl OpenDPPlugin for NoiseShim {
    const NAME: &'static str = "noise";
    fn function_options() -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: Self::NAME,
            ..Default::default()
        }
    }
}

/// Arguments for the noise expression
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct NoisePlugin {
    /// The distribution to sample from
    pub distribution: Distribution,

    /// The scale of the noise
    pub scale: f64,

    /// Distinguish between integer or floating-point support.
    pub support: Support,
}

// allow the NoiseArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for NoisePlugin {
    // makes it possible to downcast the AnonymousFunction trait object back to NoiseArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        noise_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            noise_plugin_type_udf(fields)
        }))
    }
}

impl OpenDPPlugin for NoisePlugin {
    const NAME: &'static str = "noise_plugin";
    fn function_options() -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: Self::NAME,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Distribution {
    Laplace,
    Gaussian,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Support {
    Integer,
    Float,
}

pub trait NoiseExprMeasure: 'static + Measure<Distance = f64> {
    type Metric: 'static + Metric<Distance = f64>;
    const DISTRIBUTION: Distribution;
    fn map_function<T: Number>(scale: f64)
        -> Fallible<impl Fn(&f64) -> Fallible<f64> + 'static + Send + Sync>;
}
impl NoiseExprMeasure for MaxDivergence {
    type Metric = L1Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Laplace;

    fn map_function<T: Number>(scale: f64) -> Fallible<impl Fn(&f64) -> Fallible<f64>> {
        make_laplace(
            VectorDomain::new(AtomDomain::<T>::default()),
            L1Distance::default(),
            scale,
            None
        )
    }
}
impl NoiseExprMeasure for ZeroConcentratedDivergence {
    type Metric = L2Distance<f64>;
    const DISTRIBUTION: Distribution = Distribution::Gaussian;
    fn map_function(scale: f64) -> Fallible<impl Fn(&f64) -> Fallible<f64>> {
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
pub fn make_expr_noise<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<MI>, MO>>
where
    Expr: StableExpr<PartitionDistance<MI>, MO::Metric>,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
    (ExprDomain, MO::Metric): MetricSpace,
{
    let Some((input, distribution, scale)) = match_noise_shim(&expr)? else {
        return fallible!(MakeMeasurement, "Expected noise function");
    };

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_none() && global_scale.is_none() {
        return fallible!(
            MakeMeasurement,
            "Noise mechanism requires either a scale to be set on the expression or a param to be passed to the constructor"
        );
    }

    let scale = match scale {
        Some(scale) => scale,
        None => approximate_c_stability(&t_prior)?,
    };
    let global_scale = global_scale.unwrap_or(1.);
    let scale = scale.inf_mul(&global_scale)?;
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "noise scale must not be negative");
    }

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

    let support = match &middle_domain.active_series()?.field.dtype {
        dt if dt.is_integer() => Support::Integer,
        dt if dt.is_float() => Support::Float,
        dt => {
            return fallible!(
                MakeMeasurement,
                "expected numeric data type, found {:?}",
                dt
            )
        }
    };

    // make_noise(VectorDomain::default(), middle_metric, ZExpFamily {
    //     scale: 
    // })

    let m_noise = Measurement::<_, _, MO::Metric, _>::new(
        middle_domain,
        Function::then_expr(move |input_expr| {
            apply_plugin(
                input_expr,
                expr.clone(),
                NoisePlugin {
                    scale,
                    distribution: MO::DISTRIBUTION,
                    support,
                },
            )
        }),
        middle_metric,
        MO::default(),
        PrivacyMap::new_fallible(MO::map_function(scale)?),
    )?;

    t_prior >> m_noise
}

/// Determine if the given expression is a noise expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the Noise expression and optional scale of noise
pub(crate) fn match_noise_shim(
    expr: &Expr,
) -> Fallible<Option<(&Expr, Option<Distribution>, Option<f64>)>> {
    let Some(input) = match_plugin::<NoiseShim>(expr)? else {
        return Ok(None);
    };

    let Ok([data, distribution, scale]) = <&[_; 3]>::try_from(input.as_slice()) else {
        return fallible!(MakeMeasurement, "Noise expects three input expressions");
    };

    let distribution = if let Some(dist) = literal_value_of::<String>(distribution)? {
        let dist = Distribution::deserialize(dist.into_deserializer())
            .map_err(|e: Error| err!(FailedFunction, "{:?}", e))?;
        Some(dist)
    } else {
        None
    };

    let scale = literal_value_of::<f64>(scale)?;

    Ok(Some((data, distribution, scale)))
}

// Code comment, not documentation:
// When using the plugin API from other languages, the NoiseArgs struct is serialized inside a FunctionExpr::FfiPlugin.
// When using the Rust API directly, the NoiseArgs struct is stored inside an AnonymousFunction.

/// Implementation of the noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn noise_udf(inputs: &[Series], kwargs: NoisePlugin) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "noise expects a single input expression");
    };

    let NoisePlugin {
        distribution,
        scale,
        ..
    } = kwargs;

    use DataType::*;
    match series.dtype() {
        UInt32 => noise_impl::<u32>(series, scale, distribution),
        UInt64 => noise_impl::<u64>(series, scale, distribution),
        Int8 => noise_impl::<i8>(series, scale, distribution),
        Int16 => noise_impl::<i16>(series, scale, distribution),
        Int32 => noise_impl::<i32>(series, scale, distribution),
        Int64 => noise_impl::<i64>(series, scale, distribution),
        Float32 => noise_impl::<f32>(series, scale, distribution),
        Float64 => noise_impl::<f64>(series, scale, distribution),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type, found {}", dtype),
    }
}

// PT stands for Polars Type
fn noise_impl<T: NumericDataType>(
    series: &Series,
    scale: f64,
    distribution: Distribution,
) -> PolarsResult<Series>
where
    T: Number,
    T::NumericPolars: PolarsNumericType,
    // must be able to construct the chunked array corresponding to the physical dtype from an iterator
    <T::NumericPolars as PolarsDataType>::Array: ArrayFromIter<T> + ArrayFromIter<Option<T>>,
    // must be able to convert the chunked array to a series
    ChunkedArray<T::NumericPolars>: IntoSeries,
    VectorDomain<AtomDomain<T>>: LaplaceDomain<Carrier=Vec<T>> + GaussianDomain<ZeroConcentratedDivergence, T, Carrier=Vec<T>>,
    (VectorDomain<AtomDomain<T>>, <VectorDomain<AtomDomain<T>> as LaplaceDomain>::InputMetric): MetricSpace,
    (VectorDomain<AtomDomain<T>>, <VectorDomain<AtomDomain<T>> as GaussianDomain<ZeroConcentratedDivergence, T>>::InputMetric): MetricSpace,

    RBig: TryFrom<T>,
{
    let m_noise = make_laplace(VectorDomain::new(AtomDomain::<T>::default()), Default::default(), scale, None)?;
    Ok(series
        // .i32()?.to_vec_null_aware()
        // unpack the series into a chunked array
        .unpack::<T::NumericPolars>()?
        .downcast_iter()
        .map(|chunk| {
            m_noise.invoke(&chunk.values().to_vec())
        })
        // convert the resulting chunked array back to a series
        .into_series())
}

#[cfg(feature = "ffi")]
#[polars_expr(output_type=Null)]
fn noise(_: &[Series]) -> PolarsResult<Series> {
    polars_bail!(InvalidOperation: "OpenDP expressions must be passed through make_private_lazyframe to be executed.")
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn noise_plugin_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
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
#[polars_expr(output_type_func=noise_plugin_type_udf)]
fn noise_plugin(inputs: &[Series], kwargs: NoisePlugin) -> PolarsResult<Series> {
    noise_udf(inputs, kwargs)
}
