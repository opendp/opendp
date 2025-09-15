use std::sync::Arc;

use crate::{
    core::Function,
    domains::ExprPlan,
    error::Fallible,
    interactive::{Answer, Query, Queryable},
    measurements::{
        expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
        expr_dp_frame_len::DPFrameLenShim,
        expr_dp_mean::DPMeanShim,
        expr_dp_median::DPMedianShim,
        expr_dp_quantile::DPQuantileShim,
        expr_dp_sum::DPSumShim,
        expr_noise::NoiseShim,
        expr_noisy_max::NoisyMaxShim,
    },
};
use polars::{
    frame::DataFrame,
    lazy::frame::LazyFrame,
    prelude::{AnyValue, DslPlan, GetOutput, LazySerde, NULL, len, repeat},
    series::Series,
};
#[cfg(feature = "ffi")]
use polars_plan::dsl::FunctionExpr;
use polars_plan::{
    dsl::{ColumnsUdf, Expr, SpecialEq, lit},
    plans::{LiteralValue, Null},
    prelude::FunctionOptions,
};
#[cfg(feature = "ffi")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

// this trait is used to make the Deserialize trait bound conditional on the feature flag
#[cfg(not(feature = "ffi"))]
pub(crate) trait OpenDPPlugin: 'static + Clone + ColumnsUdf {
    const NAME: &'static str;
    const SHIM: bool = false;
    fn function_options() -> FunctionOptions;
    fn get_output(&self) -> Option<GetOutput>;
}
#[cfg(feature = "ffi")]
pub(crate) trait OpenDPPlugin:
    'static + Clone + ColumnsUdf + for<'de> Deserialize<'de> + Serialize
{
    const NAME: &'static str;
    const SHIM: bool = false;
    fn function_options() -> FunctionOptions;
    fn get_output(&self) -> Option<GetOutput>;
}

#[cfg(feature = "ffi")]
static OPENDP_LIB_NAME: &str = "opendp";

pub(crate) fn match_plugin<'e, KW>(expr: &'e Expr) -> Fallible<Option<&'e Vec<Expr>>>
where
    KW: OpenDPPlugin,
{
    Ok(Some(match expr {
        #[cfg(feature = "ffi")]
        Expr::Function {
            input,
            function:
                FunctionExpr::FfiPlugin {
                    lib,
                    symbol,
                    kwargs, // Don't un-pickle! subjects the library to arbitrary code execution.
                    ..
                },
            ..
        } => {
            // check that the plugin is from the opendp library and the plugin has a matching name
            if !lib.contains(OPENDP_LIB_NAME) || symbol.as_str() != KW::NAME {
                return Ok(None);
            }

            if !kwargs.is_empty() {
                return fallible!(
                    FailedFunction,
                    "OpenDP does not allow pickled keyword arguments as they may enable remote code execution."
                );
            }

            input
        }
        Expr::AnonymousFunction {
            input, function, ..
        } => {
            if function
                .clone()
                .materialize()?
                .as_any()
                .downcast_ref::<KW>()
                .is_none()
            {
                return Ok(None);
            };
            input
        }
        _ => return Ok(None),
    }))
}

pub(crate) fn match_trusted_plugin<'e, KW>(expr: &'e Expr) -> Fallible<Option<(&'e Vec<Expr>, KW)>>
where
    KW: OpenDPPlugin,
{
    Ok(Some(match expr {
        #[cfg(feature = "ffi")]
        Expr::Function {
            input,
            function:
                FunctionExpr::FfiPlugin {
                    lib,
                    symbol,
                    kwargs,
                    ..
                },
            ..
        } => {
            // check that the plugin is from the opendp library and the plugin has a matching name
            if !lib.contains(OPENDP_LIB_NAME) || symbol.as_str() != KW::NAME {
                return Ok(None);
            }
            let args = serde_pickle::from_slice(kwargs.as_ref(), Default::default())
                .map_err(|e| err!(FailedFunction, "{}", e))?;
            (input, args)
        }
        Expr::AnonymousFunction {
            input, function, ..
        } => {
            let function = function.clone().materialize()?;
            let Some(args) = function.as_any().downcast_ref::<KW>() else {
                return Ok(None);
            };
            (input, args.clone())
        }
        _ => return Ok(None),
    }))
}

/// Match a shim plugin with a variadic number of arguments.
///
/// # Arguments
/// * `expr` - The expression to match over
///
/// # Returns
/// The input to the expression
pub(crate) fn match_shim<P: OpenDPPlugin, const V: usize>(
    expr: &Expr,
) -> Fallible<Option<[Expr; V]>> {
    let Some(input) = match_plugin::<P>(expr)? else {
        return Ok(None);
    };

    if input.len() > V {
        return fallible!(
            MakeMeasurement,
            "{} expects no more than {V} arguments",
            P::NAME
        );
    }

    let input = [input.clone(), vec![lit(NULL); V - input.len()]].concat();
    // NOTE: once generic parameters may be used in const expressions (compiler limitation)
    //       then const V can be made an associated const on OpenDPPlugin
    let args = <[_; V]>::try_from(input).expect("input always has expected length");

    Ok(Some(args))
}

/// Augment the input expression to apply the plugin expression.
///
/// # Arguments
/// * `input_expr` - The input expression to which the plugin will be applied
/// * `plugin_expr` - A plugin expression. The input to the plugin is replaced with input_expr.
/// * `kwargs_new` - Extra parameters to the plugin
pub(crate) fn apply_plugin<KW: OpenDPPlugin>(
    input_exprs: Vec<Expr>,
    plugin_expr: Expr,
    kwargs_new: KW,
) -> Expr {
    match plugin_expr {
        // handle the case where the expression is an FFI plugin
        #[cfg(feature = "ffi")]
        Expr::Function {
            input: _, // ignore the input, as it is replaced with input_expr
            function,
        } => {
            let lib = if let Ok(path) = std::env::var("OPENDP_POLARS_LIB_PATH") {
                path.into()
            } else if let FunctionExpr::FfiPlugin { lib, .. } = function {
                lib
            } else {
                unreachable!("plugin expressions are always an FfiPlugin")
            };

            Expr::Function {
                input: input_exprs,
                function: FunctionExpr::FfiPlugin {
                    flags: KW::function_options(),
                    lib,
                    symbol: KW::NAME.into(),
                    kwargs: if KW::SHIM {
                        Default::default()
                    } else {
                        serde_pickle::to_vec(&kwargs_new, Default::default())
                            .expect("pickling does not fail")
                            .into()
                    },
                },
            }
        }
        // handle the case where the expression is an AnonymousFunction from Rust
        Expr::AnonymousFunction { .. } => Expr::AnonymousFunction {
            input: input_exprs,
            fmt_str: Box::new(KW::NAME.into()),
            output_type: kwargs_new.get_output().unwrap(),
            function: LazySerde::Deserialized(SpecialEq::new(Arc::new(kwargs_new))),
            options: KW::function_options(),
        },
        _ => unreachable!("only called after constructor checks"),
    }
}

pub(crate) fn apply_anonymous_function<KW: OpenDPPlugin>(input: Vec<Expr>, kwargs: KW) -> Expr {
    Expr::AnonymousFunction {
        input,
        fmt_str: Box::new(KW::NAME.into()),
        // pass through the constructor to activate the expression
        function: LazySerde::Deserialized(SpecialEq::new(Arc::new(kwargs.clone()))),
        // have no option but to panic in this case, since the polars api does not accept results
        output_type: kwargs
            .get_output()
            .ok_or_else(|| {
                err!(
                    FailedFunction,
                    "Anonymous function must have an output type"
                )
            })
            .unwrap(),
        options: KW::function_options(),
    }
}

pub(crate) fn literal_value_of<T: ExtractValue>(expr: &Expr) -> Fallible<Option<T>> {
    let Expr::Literal(literal) = expr else {
        return fallible!(FailedFunction, "Expected literal, found: {:?}", expr);
    };

    T::extract(literal.clone())
}

pub(crate) trait ExtractValue: Sized {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>>;
}
macro_rules! impl_extract_value_number {
    ($($ty:ty)+) => {$(impl ExtractValue for $ty {
        fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
            if literal.is_null() {
                return Ok(None);
            }
            Ok(Some(literal
                .to_any_value()
                .ok_or_else(|| err!(FailedFunction))?
                .try_extract()?))
        }
    })+}
}

impl_extract_value_number!(u8 u16 u32 u64 i8 i16 i32 i64 f32 f64);

impl ExtractValue for bool {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
        let any_value = literal.to_any_value().ok_or_else(|| err!(FailedFunction))?;

        if matches!(any_value, AnyValue::Null) {
            return Ok(None);
        }

        let AnyValue::Boolean(value) = any_value else {
            return fallible!(FailedFunction, "expected boolean, found {:?}", any_value);
        };

        Ok(Some(value))
    }
}

impl ExtractValue for Series {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
        if literal.is_null() {
            return Ok(None);
        }
        Ok(match literal {
            LiteralValue::Series(series) => Some((*series).clone()),
            _ => return fallible!(FailedFunction, "expected series, found: {:?}", literal),
        })
    }
}

impl ExtractValue for String {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
        if literal.is_null() {
            return Ok(None);
        }
        literal
            .extract_str()
            .map(|s| Some(s.to_string()))
            .ok_or_else(|| err!(FailedFunction, "expected String, found: {:?}", literal))
    }
}

impl Function<ExprPlan, ExprPlan> {
    /// # Proof Definition
    /// Return a Function that, when passed a plan,
    /// returns the same plan but with the expression extended via `function`.
    pub(crate) fn then_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &ExprPlan| arg.then(&function))
    }
}
impl Function<DslPlan, ExprPlan> {
    /// # Proof Definition
    /// Return a Function that, if passed a plan with a wildcard expression,
    /// returns the same plan but with `expr` expression instead.
    pub(crate) fn from_expr(expr: Expr) -> Self {
        Self::new_fallible(move |arg: &DslPlan| -> Fallible<ExprPlan> {
            Ok(ExprPlan {
                plan: arg.clone(),
                expr: expr.clone(),
                fill: None,
            })
        })
    }
}

impl<TI: 'static> Function<TI, ExprPlan> {
    /// # Proof Definition
    /// Returns a Function that specifies how to impute missing values representing empty groups.
    ///
    /// Polars only keeps non-empty groups in group-by,
    /// so this is used to fill missing values after joining with an explicit key set.
    pub(crate) fn fill_with(self, value: Expr) -> Self {
        // Without this repeat, the expression would be scalar-valued,
        // and broadcast later to the required length.
        // This would cause randomized plugins, like noise and noisy_max,
        // to only be applied to one row,
        // and the one noisy row would then be broadcast to the entire column.
        let fill = repeat(value.clone(), len());

        Self::new_fallible(move |arg: &TI| {
            let mut plan = self.eval(arg)?;
            plan.fill = Some(fill.clone());
            Ok(plan)
        })
    }
}

/// Helper trait for Rust users to access differentially private expressions.
pub trait PrivacyNamespace {
    fn dp(self) -> DPExpr;
}
impl PrivacyNamespace for Expr {
    fn dp(self) -> DPExpr {
        DPExpr(self)
    }
}

pub struct DPExpr(Expr);
impl DPExpr {
    /// Add noise to the expression.
    ///
    /// `scale` must not be negative or inf.
    /// Scale and distribution may be left None, to be filled later by [`make_private_lazyframe`].
    /// The noise distribution is chosen according to the privacy definition:
    ///    
    /// * Pure-DP: Laplace noise, where `scale` == standard_deviation / sqrt(2)
    /// * zCDP: Gaussian noise, where `scale` == standard_devation
    ///
    /// # Arguments
    /// * `scale` - Scale parameter for the noise distribution
    pub fn noise(self, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, scale], NoiseShim)
    }

    /// Compute the differentially private len (including nulls).
    ///
    /// # Arguments
    /// * `scale` - parameter for the noise distribution
    pub fn len(self, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, scale], DPLenShim)
    }

    /// Compute the differentially private count (excluding nulls).
    ///
    /// # Arguments
    /// * `scale` - parameter for the noise distribution
    pub fn count(self, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, scale], DPCountShim)
    }

    /// Compute the differentially private null count (exclusively nulls).
    ///
    /// # Arguments
    /// * `scale` - parameter for the noise distribution
    pub fn null_count(self, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, scale], DPNullCountShim)
    }

    /// Compute the differentially private count of unique elements (including null).
    ///
    /// # Arguments
    /// * `scale` - parameter for the noise distribution
    pub fn n_unique(self, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, scale], DPNUniqueShim)
    }

    /// Compute the differentially private sum.
    ///
    /// # Arguments
    /// * `bounds` - The bounds of the input data
    /// * `scale` - parameter for the noise distribution
    pub fn sum(self, bounds: (Expr, Expr), scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_default();
        apply_anonymous_function(vec![self.0, bounds.0, bounds.1, scale], DPSumShim)
    }

    /// Compute the differentially private mean.
    ///
    /// # Arguments
    /// * `bounds` - The bounds of the input data
    /// * `scales` - relative parameter for the scale of the noise distributions
    pub fn mean(self, bounds: (Expr, Expr), scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_default();
        apply_anonymous_function(vec![self.0, bounds.0, bounds.1, scale], DPMeanShim)
    }

    /// Report the argmax or argmin after adding noise.
    ///
    /// The scale calibrates the level of entropy when selecting an index.
    ///
    /// # Arguments
    /// * `negate` - Flip signs to report noisy min.
    /// * `scale` - Noise scale parameter for the noise distribution.
    pub fn noisy_max(self, negate: bool, scale: Option<f64>) -> Expr {
        let negate = lit(negate);
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, negate, scale], NoisyMaxShim)
    }

    /// Compute a differentially private quantile.
    ///
    /// The scale calibrates the level of entropy when selecting a candidate.
    ///
    /// # Arguments
    /// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
    /// * `candidates` - Potential quantiles to select from.
    /// * `scale` - scale parameter for the noise distribution.
    pub fn quantile(self, alpha: f64, candidates: Series, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(
            vec![self.0, lit(alpha), lit(candidates), scale],
            DPQuantileShim,
        )
    }

    /// Compute a differentially private median.
    ///
    /// The scale calibrates the level of entropy when selecting a candidate.
    ///
    /// # Arguments
    /// * `candidates` - Potential quantiles to select from.
    /// * `scale` - scale parameter for the noise distribution.
    pub fn median(self, candidates: Series, scale: Option<f64>) -> Expr {
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, lit(candidates), scale], DPMedianShim)
    }
}

/// Compute the differentially private len (including nulls).
///
/// # Arguments
/// * `scale` - parameter for the noise distribution
pub fn dp_len(scale: Option<f64>) -> Expr {
    let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
    apply_anonymous_function(vec![scale], DPFrameLenShim)
}

pub enum OnceFrameQuery {
    Collect,
}

pub enum OnceFrameAnswer {
    Collect(DataFrame),
}

pub(crate) struct ExtractLazyFrame;

pub type OnceFrame = Queryable<OnceFrameQuery, OnceFrameAnswer>;

impl From<LazyFrame> for OnceFrame {
    fn from(value: LazyFrame) -> Self {
        let mut state = Some(value);
        Self::new_raw(move |_self: &Self, query: Query<OnceFrameQuery>| {
            let Some(lazyframe) = state.clone() else {
                return fallible!(FailedFunction, "OnceFrame has been exhausted");
            };
            Ok(match query {
                Query::External(q_external) => Answer::External(match q_external {
                    OnceFrameQuery::Collect => {
                        let dataframe = lazyframe.collect()?;
                        let n = dataframe.height();
                        let dataframe = dataframe.sample_n_literal(n, false, true, None)?;
                        state.take();
                        OnceFrameAnswer::Collect(dataframe)
                    }
                }),
                Query::Internal(q_internal) => Answer::Internal({
                    if q_internal.downcast_ref::<ExtractLazyFrame>().is_some() {
                        Box::new(lazyframe)
                    } else {
                        return fallible!(FailedFunction, "Unrecognized internal query");
                    }
                }),
            })
        })
    }
}

impl OnceFrame {
    pub fn collect(mut self) -> Fallible<DataFrame> {
        if let Answer::External(OnceFrameAnswer::Collect(dataframe)) =
            self.eval_query(Query::External(&OnceFrameQuery::Collect))?
        {
            Ok(dataframe)
        } else {
            // should never be reached
            fallible!(
                FailedFunction,
                "Collect returned invalid answer: Please report this bug"
            )
        }
    }

    /// Extract the compute plan with the private data.
    ///
    /// Requires "honest-but-curious" because the privacy guarantees only apply if:
    ///
    /// 1. The LazyFrame (compute plan) is only ever executed once.
    /// 2. The analyst does not observe ordering of rows in the output.
    ///    
    /// To ensure that row ordering is not observed:
    ///
    /// 1. Do not extend the compute plan with order-sensitive computations.
    /// 2. Shuffle the output once collected ([in Polars sample all, with shuffling enabled](https://docs.rs/polars/latest/polars/frame/struct.DataFrame.html#method.sample_n_literal)).
    #[cfg(feature = "honest-but-curious")]
    pub fn lazyframe(&mut self) -> LazyFrame {
        let answer = self.eval_query(Query::Internal(&ExtractLazyFrame)).unwrap();
        let Answer::Internal(boxed) = answer else {
            panic!("failed to extract");
        };
        let Ok(lazyframe) = boxed.downcast() else {
            panic!("failed to extract");
        };
        *lazyframe
    }
}

pub(crate) fn get_disabled_features_message() -> String {
    #[allow(unused_mut)]
    let mut disabled_features: Vec<&'static str> = vec![];

    #[cfg(not(feature = "contrib"))]
    disabled_features.push("contrib");
    #[cfg(not(feature = "floating-point"))]
    disabled_features.push("floating-point");
    #[cfg(not(feature = "honest-but-curious"))]
    disabled_features.push("honest-but-curious");

    if disabled_features.is_empty() {
        String::new()
    } else {
        format!(
            "This may be due to disabled features: {}. ",
            disabled_features.join(", ")
        )
    }
}
