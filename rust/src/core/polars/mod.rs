use std::sync::Arc;

use crate::{
    measurements::expr_laplace::LaplaceArgs,
    transformations::expr_discrete_quantile_score::DQScoreArgs,
};
use polars_plan::{
    dsl::{len, lit, Expr, FunctionExpr, SeriesUdf, SpecialEq},
    logical_plan::Literal,
    prelude::FunctionOptions,
};
use serde::{Deserialize, Serialize};

use super::{Fallible, Function};

// this trait is used to make the Deserialize trait bound conditional on the feature flag
#[cfg(not(feature = "ffi"))]
pub(crate) trait OpenDPPlugin: Clone + SeriesUdf {
    fn get_options(&self) -> FunctionOptions;
}
#[cfg(feature = "ffi")]
pub(crate) trait OpenDPPlugin:
    'static + Clone + SeriesUdf + for<'de> Deserialize<'de> + Serialize
{
    fn get_options(&self) -> FunctionOptions;
}

static OPENDP_LIB_NAME: &str = "opendp";

pub(crate) fn match_plugin<'e, KW>(
    expr: &'e Expr,
    name: &str,
) -> Fallible<Option<(&'e Vec<Expr>, KW)>>
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
            if !lib.contains(OPENDP_LIB_NAME) || symbol.as_ref() != name {
                return Ok(None);
            }
            let args = serde_pickle::from_slice(kwargs.as_ref(), Default::default())
                .map_err(|e| err!(FailedFunction, e.to_string()))?;
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
            if let FunctionExpr::FfiPlugin { kwargs, .. } = &mut function {
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
        options: kwargs.get_options(),
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
pub trait PrivacyNamespaceHelper {
    fn dp(self) -> DPNamespace;
}
impl PrivacyNamespaceHelper for Expr {
    fn dp(self) -> DPNamespace {
        DPNamespace(self)
    }
}

pub struct DPNamespace(Expr);
impl DPNamespace {
    /// Add Laplace noise to the expression.
    ///
    /// `scale` must not be negative or inf.
    /// Scale may be left None, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    ///
    /// # Arguments
    /// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
    pub fn laplace(self, scale: Option<f64>) -> Expr {
        apply_anonymous_function(vec![self.0], LaplaceArgs { scale })
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
            .laplace(scale)
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
    #[allow(dead_code)]
    pub(crate) fn quantile_score(self, alpha: f64, candidates: Vec<f64>) -> Expr {
        let args = DQScoreArgs {
            alpha,
            candidates,
            constants: None,
        };
        apply_anonymous_function(vec![self.0], args)
    }
}
