use std::{env, sync::Arc};

use crate::{
    core::Function,
    error::Fallible,
    interactive::{Answer, Query, Queryable},
    measurements::{
        expr_index_candidates::IndexCandidatesShim,
        expr_noise::{Distribution, NoiseShim},
        expr_report_noisy_max::ReportNoisyMaxShim,
        Optimize,
    },
    transformations::expr_discrete_quantile_score::DiscreteQuantileScoreShim,
};
use polars::{frame::DataFrame, lazy::frame::LazyFrame, prelude::NamedFrom, series::Series};
use polars_plan::{
    dsl::{len, lit, Expr, FunctionExpr, SeriesUdf, SpecialEq},
    plans::{Literal, LiteralValue, Null},
    prelude::FunctionOptions,
};
use serde::{Deserialize, Serialize};

// this trait is used to make the Deserialize trait bound conditional on the feature flag
#[cfg(not(feature = "ffi"))]
pub(crate) trait OpenDPShim: 'static + Clone + SeriesUdf {
    const NAME: &'static str;
    fn function_options() -> FunctionOptions;
}
#[cfg(feature = "ffi")]
pub(crate) trait OpenDPPlugin:
    'static + Clone + SeriesUdf + for<'de> Deserialize<'de> + Serialize
{
    const NAME: &'static str;
    fn function_options() -> FunctionOptions;
}

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
                },
            ..
        } => {
            // check that the plugin is from the opendp library and the plugin has a matching name
            if !lib.contains(OPENDP_LIB_NAME) || symbol.as_ref() != KW::NAME {
                return Ok(None);
            }

            if !kwargs.is_empty() {
                return fallible!(FailedFunction, "OpenDP does not allow pickled keyword arguments as they may enable remote code execution.");
            }

            input
        }
        Expr::AnonymousFunction {
            input, function, ..
        } => {
            if function.as_any().downcast_ref::<KW>().is_none() {
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
                },
            ..
        } => {
            // check that the plugin is from the opendp library and the plugin has a matching name
            if !lib.contains(OPENDP_LIB_NAME) || symbol.as_ref() != KW::NAME {
                return Ok(None);
            }
            let args = serde_pickle::from_slice(kwargs.as_ref(), Default::default())
                .map_err(|e| err!(FailedFunction, "{}", e))?;
            (input, args)
        }
        Expr::AnonymousFunction {
            input, function, ..
        } => {
            let Some(args) = function.as_any().downcast_ref::<KW>() else {
                return Ok(None);
            };
            (input, args.clone())
        }
        _ => return Ok(None),
    }))
}

/// Augment the input expression to apply the plugin expression.
///
/// # Arguments
/// * `input_expr` - The input expression to which the Laplace noise will be added
/// * `plugin_expr` - A plugin expression. The input to the plugin is replaced with input_expr.
/// * `kwargs` - Extra parameters to the plugin
pub(crate) fn apply_plugin<KW: OpenDPPlugin>(
    input_expr: Expr,
    plugin_expr: Expr,
    kwargs_new: KW,
) -> Expr {
    match plugin_expr {
        // handle the case where the expression is an FFI plugin
        #[cfg(feature = "ffi")]
        Expr::Function {
            input: _, // ignore the input, as it is replaced with input_expr
            mut function,
            options,
        } => {
            // overwrite the kwargs to update the noise scale parameter in the FFI plugin
            if let FunctionExpr::FfiPlugin {
                lib,
                symbol,
                kwargs,
            } = &mut function
            {
                if let Ok(path) = env::var("OPENDP_POLARS_LIB_PATH") {
                    *lib = Arc::from(path);
                }
                *symbol = KW::NAME.into();
                *kwargs = serde_pickle::to_vec(&kwargs_new, Default::default())
                    .expect("pickling does not fail")
                    .into();
            }

            Expr::Function {
                input: vec![input_expr],
                function,
                options,
            }
        }
        // handle the case where the expression is an AnonymousFunction from Rust
        Expr::AnonymousFunction {
            output_type,
            options,
            ..
        } => Expr::AnonymousFunction {
            input: vec![input_expr],
            function: SpecialEq::new(Arc::new(kwargs_new)),
            output_type,
            options,
        },
        _ => unreachable!("only called after constructor checks"),
    }
}

pub(crate) fn apply_anonymous_function<KW: OpenDPPlugin>(input: Vec<Expr>, kwargs: KW) -> Expr {
    Expr::AnonymousFunction {
        input,
        // pass through the constructor to activate the expression
        function: SpecialEq::new(Arc::new(kwargs.clone())),
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
        return fallible!(FailedFunction);
    };

    T::extract(literal.clone())
}

pub(crate) trait ExtractValue: Sized {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>>;
}
macro_rules! impl_extract_value {
    ($($ty:ty)+) => {$(impl ExtractValue for $ty {
        fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
            if let LiteralValue::Null = literal {
                return Ok(None);
            }
            Ok(Some(literal
                .to_any_value()
                .ok_or_else(|| err!(FailedFunction))?
                .try_extract()?))
        }
    })+}
}

impl_extract_value!(u32 u64 i32 i64 f32 f64);

impl ExtractValue for Series {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
        Ok(match literal {
            LiteralValue::Null => None,
            LiteralValue::Series(series) => Some((*series).clone()),
            _ => return fallible!(FailedFunction, "expected series, found: {:?}", literal),
        })
    }
}

impl ExtractValue for String {
    fn extract(literal: LiteralValue) -> Fallible<Option<Self>> {
        Ok(match literal {
            LiteralValue::Null => None,
            LiteralValue::String(string) => Some(string),
            _ => return fallible!(FailedFunction, "expected String, found: {:?}", literal),
        })
    }
}

pub(crate) trait ExprFunction {
    fn then_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self;
    fn from_expr(expr: Expr) -> Self;
}

impl<F: Clone> ExprFunction for Function<(F, Expr), (F, Expr)> {
    fn then_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| (arg.0.clone(), function(arg.1.clone())))
    }
    fn from_expr(expr: Expr) -> Self {
        Self::new_fallible(
            move |(frame, expr_wild): &(F, Expr)| -> Fallible<(F, Expr)> {
                assert_is_wildcard(expr_wild)?;
                Ok((frame.clone(), expr.clone()))
            },
        )
    }
}
impl<F: Clone> ExprFunction for Function<(F, Expr), Expr> {
    fn then_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| function(arg.1.clone()))
    }
    fn from_expr(expr: Expr) -> Self {
        Self::new_fallible(move |(_, expr_wild): &(F, Expr)| -> Fallible<Expr> {
            assert_is_wildcard(expr_wild)?;
            Ok(expr.clone())
        })
    }
}

impl ExprFunction for Function<Expr, Expr> {
    fn then_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &Expr| function(arg.clone()))
    }
    fn from_expr(expr: Expr) -> Self {
        Self::new_fallible(move |expr_wild: &Expr| -> Fallible<Expr> {
            assert_is_wildcard(expr_wild)?;
            Ok(expr.clone())
        })
    }
}

fn assert_is_wildcard(expr: &Expr) -> Fallible<()> {
    if expr != &Expr::Wildcard {
        return fallible!(
            FailedFunction,
            "The only valid input expression is all() (denoting that all columns are selected)."
        );
    }
    Ok(())
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
    /// If distribution is None, then the noise distribution will be chosen for you:
    ///    
    /// * Pure-DP: Laplace noise, where `scale` == standard_deviation / sqrt(2)
    /// * zCDP: Gaussian noise, where `scale` == standard_devation
    ///
    /// # Arguments
    /// * `scale` - Scale parameter for the noise distribution
    /// * `distribution` - Either Laplace, Gaussian or None.
    pub fn noise(self, distribution: Option<Distribution>, scale: Option<f64>) -> Expr {
        let distribution = distribution
            .map(|d| lit(format!("{:?}", d)))
            .unwrap_or_else(|| lit(Null {}));
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, distribution, scale], NoiseShim)
    }

    /// Add Laplace noise to the expression.
    ///
    /// `scale` must not be negative or inf.
    /// Scale may be left None, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    ///
    /// # Arguments
    /// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    pub fn laplace(self, scale: Option<f64>) -> Expr {
        self.noise(Some(Distribution::Laplace), scale)
    }

    /// Add Gaussian noise to the expression.
    ///
    /// `scale` must not be negative or inf.
    /// Scale may be left None, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    ///
    /// # Arguments
    /// * `scale` - Noise scale parameter for the Gaussian distribution. `scale` == standard_deviation.
    pub fn gaussian(self, scale: Option<f64>) -> Expr {
        self.noise(Some(Distribution::Gaussian), scale)
    }

    /// Compute the differentially private sum.
    ///
    /// # Arguments
    /// * `bounds` - The bounds of the input data.
    /// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    pub fn sum<L: Literal>(self, bounds: (L, L), scale: Option<f64>) -> Expr {
        self.0
            .clip(lit(bounds.0), lit(bounds.1))
            .sum()
            .dp()
            .noise(None, scale)
    }

    /// Compute the differentially private mean.
    ///
    /// The scale calibrates the amount of noise to be added to the sum.
    ///
    /// # Arguments
    /// * `bounds` - The bounds of the input data.
    /// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    pub fn mean<L: Literal>(self, bounds: (L, L), scale: Option<f64>) -> Expr {
        self.0.dp().sum(bounds, scale) / len()
    }

    /// Score the utility of each candidate for representing the true quantile.
    ///
    /// Candidates closer to the true quantile are assigned scores closer to zero.
    /// Lower scores are better.
    ///
    /// # Arguments
    /// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
    /// * `candidates` - Set of possible quantiles to evaluate the utility of.
    pub(crate) fn quantile_score(self, alpha: f64, candidates: Series) -> Expr {
        apply_anonymous_function(
            vec![self.0, lit(alpha), lit(candidates)],
            DiscreteQuantileScoreShim,
        )
    }

    /// Report the argmax or argmin after adding Gumbel noise.
    ///
    /// The scale calibrates the level of entropy when selecting an index.
    ///
    /// # Arguments
    /// * `optimize` - Distinguish between argmax and argmin.
    /// * `scale` - Noise scale parameter for the Gumbel distribution.
    pub(crate) fn report_noisy_max_gumbel(self, optimize: Optimize, scale: Option<f64>) -> Expr {
        let optimize = lit(format!("{optimize}"));
        let scale = scale.map(lit).unwrap_or_else(|| lit(Null {}));
        apply_anonymous_function(vec![self.0, optimize, scale], ReportNoisyMaxShim)
    }

    /// Index into a candidate set.
    ///
    /// Typically used after `rnm_gumbel` to map selected indices to candidates.
    ///
    /// # Arguments
    /// * `candidates` - The values that each selected index corresponds to.
    pub(crate) fn index_candidates(self, candidates: Series) -> Expr {
        apply_anonymous_function(vec![self.0, lit(candidates)], IndexCandidatesShim)
    }

    /// Compute a differentially private quantile.
    ///
    /// The scale calibrates the level of entropy when selecting a candidate.
    ///
    /// # Arguments
    /// * `alpha` - a value in $[0, 1]$. Choose 0.5 for median
    /// * `candidates` - Potential quantiles to select from.
    /// * `scale` - Noise scale parameter for the Gumbel distribution.
    pub fn quantile(self, alpha: f64, candidates: Series, scale: Option<f64>) -> Expr {
        self.0
            .dp()
            .quantile_score(alpha, candidates.clone())
            .dp()
            .report_noisy_max_gumbel(Optimize::Min, scale)
            .dp()
            .index_candidates(Series::new("", candidates))
    }

    /// Compute a differentially private median.
    ///
    /// The scale calibrates the level of entropy when selecting a candidate.
    ///
    /// # Arguments
    /// * `candidates` - Potential quantiles to select from.
    /// * `scale` - Noise scale parameter for the Gumbel distribution.
    pub fn median(self, candidates: Series, scale: Option<f64>) -> Expr {
        self.0.dp().quantile(0.5, candidates, scale)
    }
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
    /// 1. The LazyFrame (compute plan) is only ever executed once.
    /// 2. The analyst does not observe ordering of rows in the output. To ensure this, shuffle the output.
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
