use crate::core::{
    apply_plugin, match_plugin, ExprFunction, Metric, MetricSpace, OpenDPPlugin, PrivacyMap, Scalar,
};
use crate::measurements::{get_discretization_consts, laplace_map};
use crate::metrics::L1Distance;
use crate::traits::samplers::{sample_discrete_laplace, sample_discrete_laplace_Z2k};
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
use polars::error::{polars_bail, polars_err};
use polars::error::{PolarsError, PolarsResult};
use polars::lazy::dsl::Expr;
use polars::series::{IntoSeries, Series};
use polars_plan::dsl::{GetOutput, SeriesUdf};
use polars_plan::prelude::{ApplyOptions, FunctionOptions};
use pyo3_polars::derive::polars_expr;
use serde::{Deserialize, Serialize};

/// Make a measurement that adds Laplace noise to a Polars expression.
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `expr` - The expression to which the Laplace noise will be added
/// * `param` - (Re)scale the noise parameter for the laplace distribution
pub fn make_expr_laplace<MI: 'static + Metric>(
    input_domain: ExprDomain,
    input_metric: MI,
    expr: Expr,
    param: f64,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MaxDivergence<f64>>>
where
    Expr: StableExpr<MI, L1Distance<Scalar>>,
    (ExprDomain, MI): MetricSpace,
{
    let (input, scale) =
        match_laplace(&expr)?.ok_or_else(|| err!(MakeMeasurement, "Expected Laplace function"))?;

    let t_prior = input
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    if scale.is_nan() && param.is_nan() {
        return fallible!(
            MakeMeasurement,
            "Laplace mechanism requires either a scale to be set on the expression or a param to be passed to the constructor"
        );
    }

    let scale = if scale.is_nan() { 1. } else { scale };
    let param = if param.is_nan() { 1. } else { param };
    let scale = scale.inf_mul(&param)?;

    if middle_domain.active_series()?.nullable {
        return fallible!(
            MakeMeasurement,
            "Laplace mechanism requires non-nullable input"
        );
    }

    t_prior
        >> Measurement::<_, _, L1Distance<Scalar>, _>::new(
            middle_domain,
            Function::new_expr(move |input_expr| {
                apply_plugin(input_expr, expr.clone(), LaplaceArgs { scale })
            }),
            middle_metric,
            MaxDivergence::default(),
            PrivacyMap::new_fallible(move |d_in: &Scalar| laplace_map(scale, 0.)(&d_in.f64()?)),
        )?
}

/// Determine if the given expression is a Laplace noise expression.
///
/// # Arguments
/// * `expr` - The expression to check
///
/// # Returns
/// The input to the Laplace expression and the scale of the Laplace noise
pub(crate) fn match_laplace(expr: &Expr) -> Fallible<Option<(&Expr, f64)>> {
    let Some((input, LaplaceArgs { scale })) = match_plugin(expr, "laplace")? else {
        return Ok(None);
    };

    let [input] = <&[_; 1]>::try_from(input.as_slice())
        .map_err(|_| err!(MakeMeasurement, "Laplace expects a single input expression"))?;

    Ok(Some((input, scale)))
}

// When using the plugin API from other languages, the LaplaceArgs struct is serialized inside a FunctionExpr::FfiPlugin.
// When using the Rust API directly, the LaplaceArgs struct is stored inside an AnonymousFunction.
//
/// Arguments for the Laplace noise expression
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct LaplaceArgs {
    /// The scale of the Laplace noise.
    ///
    /// Scale may be left NaN, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    pub scale: f64,
}

impl OpenDPPlugin for LaplaceArgs {
    fn get_options(&self) -> FunctionOptions {
        FunctionOptions {
            collect_groups: ApplyOptions::ElementWise,
            fmt_str: "laplace",
            ..Default::default()
        }
    }
}

// allow the LaplaceArgs struct to be stored inside an AnonymousFunction, when used from Rust directly
impl SeriesUdf for LaplaceArgs {
    // makes it possible to downcast the AnonymousFunction trait object back to LaplaceArgs
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call_udf(&self, s: &mut [Series]) -> PolarsResult<Option<Series>> {
        laplace_udf(s, self.clone()).map(Some)
    }

    fn get_output(&self) -> Option<GetOutput> {
        Some(GetOutput::map_fields(|fields| {
            laplace_type_udf(fields)
                // NOTE: it would be better if this didn't need to fall back,
                // but Polars does not support raising an error
                .ok()
                .unwrap_or_else(|| fields[0].clone())
        }))
    }
}

/// Implementation of the Laplace noise expression.
///
/// The Polars engine executes this function over chunks of data.
fn laplace_udf(inputs: &[Series], kwargs: LaplaceArgs) -> PolarsResult<Series> {
    let Ok([series]) = <&[_; 1]>::try_from(inputs) else {
        polars_bail!(InvalidOperation: "Laplace expects a single input field");
    };
    let scale = kwargs.scale;

    // PT stands for Polars Type
    fn laplace_impl_integer<PT: 'static + PolarsDataType>(
        series: &Series,
        scale: f64,
    ) -> PolarsResult<Series>
    where
        // the physical (rust) dtype must be able to be converted into a big integer
        for<'a> IBig: From<PT::Physical<'a>>,
        // a big integer must be able to be converted into the physical (rust) dtype
        for<'a> PT::Physical<'a>: SaturatingCast<IBig>,
        // must be able to construct the chunked array corresponding to the physical dtype from an iterator
        for<'a> PT::Array:
            ArrayFromIter<PT::Physical<'a>> + ArrayFromIter<Option<PT::Physical<'a>>>,
        // must be able to convert the chunked array to a series
        ChunkedArray<PT>: IntoSeries,
    {
        let scale = RBig::try_from(scale)
            .map_err(|_| PolarsError::ComputeError("scale must be finite".into()))?;
        Ok(series
            // unpack the series into a chunked array
            .unpack::<PT>()?
            // apply Laplace noise to the non-null values
            .try_apply_nonnull_values_generic(|v| {
                let v = IBig::from(v);
                sample_discrete_laplace(scale.clone())
                    .map(|noise| PT::Physical::saturating_cast(v + noise))
            })?
            // convert the resulting chunked array back to a series
            .into_series())
    }

    // PT stands for Polars Type
    fn laplace_impl_float<PT: 'static + PolarsDataType>(
        series: &Series,
        scale: f64,
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
            // apply Laplace noise to the non-null values
            .try_apply_nonnull_values_generic(|v| sample_discrete_laplace_Z2k(v, scale, k))?
            // convert the resulting chunked array back to a series
            .into_series())
    }

    use DataType::*;
    match series.dtype() {
        UInt32 => laplace_impl_integer::<UInt32Type>(series, scale),
        UInt64 => laplace_impl_integer::<UInt64Type>(series, scale),
        Int8 => laplace_impl_integer::<Int8Type>(series, scale),
        Int16 => laplace_impl_integer::<Int16Type>(series, scale),
        Int32 => laplace_impl_integer::<Int32Type>(series, scale),
        Int64 => laplace_impl_integer::<Int64Type>(series, scale),
        Float32 => laplace_impl_float::<Float32Type>(series, scale),
        Float64 => laplace_impl_float::<Float64Type>(series, scale),
        UInt8 | UInt16 => {
            polars_bail!(InvalidOperation: "u8 and u16 not supported in the OpenDP Polars plugin. Please use u32 or u64.")
        }
        dtype => polars_bail!(InvalidOperation: "Expected numeric data type, found {}", dtype),
    }
}

/// Helper function for the Polars plan optimizer to determine the output type of the expression.
///
/// Ensures that the input field is numeric.
pub(crate) fn laplace_type_udf(input_fields: &[Field]) -> PolarsResult<Field> {
    let Ok([field]) = <&[Field; 1]>::try_from(input_fields) else {
        polars_bail!(InvalidOperation: "laplace expects a single input field")
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

// generate the FFI plugin for the Laplace noise expression
#[cfg(feature = "ffi")]
#[polars_expr(output_type_func=laplace_type_udf)]
fn laplace(inputs: &[Series], kwargs: LaplaceArgs) -> PolarsResult<Series> {
    laplace_udf(inputs, kwargs)
}

#[cfg(test)]
mod test_make_laplace_expr {
    use super::*;
    use polars::prelude::*;

    use crate::{
        core::PrivacyNamespaceHelper,
        measurements::{make_private_expr, make_private_lazyframe},
        metrics::{PartitionDistance, SymmetricDistance},
        transformations::polars_test::get_test_data,
    };

    #[test]
    fn test_make_expr_laplace() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.select();
        let scale: f64 = 0.0;

        let m_quant = make_private_expr(
            expr_domain,
            PartitionDistance(SymmetricDistance),
            MaxDivergence::default(),
            col("B").dp().sum((0., 1.), scale),
            scale,
        )?;

        let dp_expr = m_quant.invoke(&(lf.logical_plan.clone(), all()))?;

        let df_actual = lf.clone().select([dp_expr]).collect()?;

        println!("{:?}", df_actual);

        Ok(())
    }

    #[test]
    fn test_make_laplace_grouped() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let scale: f64 = 0.0;

        let expr_exp = col("B").clip(lit(1), lit(2)).sum().dp().laplace(f64::NAN);
        let m_lap = make_private_lazyframe(
            lf_domain,
            SymmetricDistance,
            MaxDivergence::default(),
            lf.clone().group_by(["A"]).agg([expr_exp]),
            scale,
        )?;

        let df_act = m_lap.invoke(&lf)?.sort("A", Default::default()).collect()?;
        let df_exp = lf
            .clone()
            .group_by(&[col("A")])
            .agg([sum("B")])
            .sort("A", Default::default())
            .collect()?;

        assert_eq!(
            df_act.sort(["A"], false, false)?,
            df_exp.sort(["A"], false, false)?
        );
        Ok(())
    }
}
