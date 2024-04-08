use std::sync::Arc;

use crate::{
    measurements::{
        expr_laplace::LaplaceArgs, expr_report_noisy_max_gumbel::RNMGumbelArgs, Optimize,
    },
    traits::InfCast,
    transformations::expr_discrete_quantile_score::DQScoreArgs,
};
use polars_plan::{
    dsl::{lit, Expr, FunctionExpr, SeriesUdf, SpecialEq},
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

pub trait ExprFunction {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self;
}

impl<F: Clone> ExprFunction for Function<(F, Expr), (F, Expr)> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| (arg.0.clone(), function(arg.1.clone())))
    }
}
impl<F: Clone> ExprFunction for Function<(F, Expr), Expr> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &(F, Expr)| function(arg.1.clone()))
    }
}

impl ExprFunction for Function<Expr, Expr> {
    fn new_expr(function: impl Fn(Expr) -> Expr + 'static + Send + Sync) -> Self {
        Self::new(move |arg: &Expr| function(arg.clone()))
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
pub enum Scalar {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl Scalar {
    pub fn f64(&self) -> Fallible<f64> {
        match self {
            Scalar::I32(v) => f64::inf_cast(*v),
            Scalar::I64(v) => f64::inf_cast(*v),
            Scalar::U32(v) => f64::inf_cast(*v),
            Scalar::U64(v) => f64::inf_cast(*v),
            Scalar::F32(v) => f64::inf_cast(*v),
            Scalar::F64(v) => f64::inf_cast(*v),
        }
    }
}

impl PartialOrd for Scalar {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Scalar::I32(l), Scalar::I32(r)) => l.partial_cmp(r),
            (Scalar::I64(l), Scalar::I64(r)) => l.partial_cmp(r),
            (Scalar::U32(l), Scalar::U32(r)) => l.partial_cmp(r),
            (Scalar::U64(l), Scalar::U64(r)) => l.partial_cmp(r),
            (Scalar::F32(l), Scalar::F32(r)) => l.partial_cmp(r),
            (Scalar::F64(l), Scalar::F64(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

macro_rules! impl_from_for_scalar {
    ($t:ty, $s:ident) => {
        impl From<$t> for Scalar {
            fn from(value: $t) -> Self {
                Scalar::$s(value)
            }
        }
    };
}
impl_from_for_scalar!(i32, I32);
impl_from_for_scalar!(i64, I64);
impl_from_for_scalar!(u32, U32);
impl_from_for_scalar!(u64, U64);
impl_from_for_scalar!(f32, F32);
impl_from_for_scalar!(f64, F64);

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
    /// `scale` must not be negative or inf.
    /// Scale may be left NaN, to be filled later by [`make_private_expr`] or [`make_private_lazyframe`].
    pub fn laplace(self, scale: f64) -> Expr {
        apply_anonymous_function(vec![self.0], LaplaceArgs { scale })
    }

    pub fn sum<L: Literal>(self, bounds: (L, L), scale: f64) -> Expr {
        self.0
            .clip(lit(bounds.0), lit(bounds.1))
            .sum()
            .dp()
            .laplace(scale)
    }

    pub fn mean<L: Literal>(self, bounds: (L, L), scale: f64) -> Expr {
        self.0.clone().dp().sum(bounds, scale) / self.0.len()
    }

    pub fn quantile_score(self, candidates: Vec<f64>, alpha: f64) -> Expr {
        let args = DQScoreArgs {
            alpha,
            candidates,
            constants: None,
        };
        apply_anonymous_function(vec![self.0], args)
    }

    pub(crate) fn rnm_gumbel(self, scale: f64, optimize: Optimize) -> Expr {
        let optimize = match optimize {
            Optimize::Min => "min",
            Optimize::Max => "max",
        }
        .to_string();
        apply_anonymous_function(vec![self.0], RNMGumbelArgs { scale, optimize })
    }

    pub fn quantile(self, candidates: Vec<f64>, alpha: f64, scale: f64) -> Expr {
        self.0
            .dp()
            .quantile_score(candidates, alpha)
            .dp()
            .rnm_gumbel(scale, Optimize::Min)
    }
}
