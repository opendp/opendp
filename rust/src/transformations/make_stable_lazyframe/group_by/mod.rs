use std::collections::HashSet;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, DslPlanDomain, FrameDomain, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{Bound, Bounds, FrameDistance, L0PInfDistance, L01InfDistance};
use crate::polars::options::ORDINARY_GROUP_BY_OPTIONS;
use crate::polars::permute::match_permute;
use crate::traits::{InfMul, option_min};
use crate::transformations::StableExpr;
use crate::transformations::make_stable_expr::cast::{check_private_cast, is_numeric_coercion};
use crate::transformations::traits::UnboundedMetric;
use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::plans::{DynLiteralValue, LiteralValue};

use super::StableDslPlan;

#[cfg(test)]
mod test;

/// Transformation for stable group-by and aggregate.
///
/// # Arguments
/// * `input_domain` - The domain of the input LazyFrame.
/// * `input_metric` - The metric of the input LazyFrame.
/// * `plan` - The LazyFrame to transform.
pub fn make_stable_group_by<M: UnboundedMetric>(
    input_domain: DslPlanDomain,
    input_metric: FrameDistance<M>,
    plan: DslPlan,
) -> Fallible<Transformation<DslPlanDomain, FrameDistance<M>, DslPlanDomain, FrameDistance<M>>> {
    if M::ORDERED {
        return fallible!(
            MakeTransformation,
            "Sample-and-aggregate requires an unordered input metric."
        );
    }
    #[cfg(patch_polars)]
    let DslPlan::GroupBy {
        input,
        keys,
        predicates,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected group-by in logical plan");
    };
    #[cfg(not(patch_polars))]
    let DslPlan::GroupBy {
        input,
        keys,
        aggs,
        apply,
        maintain_order,
        options,
    } = plan
    else {
        return fallible!(MakeTransformation, "Expected group-by in logical plan");
    };

    if apply.is_some() {
        return fallible!(
            MakeTransformation,
            "Apply is not currently supported in the logical plan. Please open an issue if this would be useful to you."
        );
    }

    #[cfg(patch_polars)]
    if !predicates.is_empty() {
        // This is equivalent to running a filter after, as far as I can tell.
        // Possibly useful to support.
        return fallible!(
            MakeTransformation,
            "Having is not currently supported in logical plan. Please open an issue if this would be useful to you."
        );
    }

    if options.as_ref() != &ORDINARY_GROUP_BY_OPTIONS {
        return fallible!(
            MakeTransformation,
            "Unsupported stable group-by options: {options:?}"
        );
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric): (_, FrameDistance<M>) = t_prior.output_space();

    // create a transformation for each expression
    let expr_domain = WildExprDomain {
        columns: middle_domain.series_domains.clone(),
        context: Context::RowByRow,
    };

    // each expression must be stable row by row
    let stable_keys = keys
        .iter()
        .map(|key| {
            let t_key: Transformation<_, _, _, L01InfDistance<M>> = key
                .clone()
                .make_stable(expr_domain.clone(), L0PInfDistance(middle_metric.0.clone()))?;
            if !canonical_sort_dtype(&t_key.output_domain.column.dtype()) {
                return fallible!(MakeTransformation,
                    "Floating-point grouping keys may emit permutation-dependent representatives (signed zeros or NaN payloads).");
            }
            Ok(t_key.invoke(&input)?.expr)
        })
        .collect::<Fallible<Vec<_>>>()?;

    // Validate fallibility and permutation invariance in the same traversal.
    let schema = middle_domain.schema();
    aggs.iter()
        .try_for_each(|e| check_permutation_invariance(e, &schema))?;

    if middle_metric.0.identifier().is_some() {
        return fallible!(
            MakeTransformation,
            "stable groupby (sample and aggregate) is not supported on datasets with unbounded row contributions. If you want to execute a groupby truncation, include the identifier in the groupby keys."
        );
    }

    // use Polars to compute the output dtype
    let series_domains = middle_domain
        .simulate_schema(|lf| lf.group_by(&stable_keys).agg(&aggs))?
        .iter_fields()
        .map(SeriesDomain::new_from_field)
        .collect::<Fallible<_>>()?;

    let h_keys = keys.iter().cloned().collect();
    let output_domain = FrameDomain::new_with_margins(
        series_domains,
        (middle_domain.margins.iter().cloned())
            .filter(|m| m.by.is_subset(&h_keys))
            .map(|mut m| {
                m.by = materialize_grouping_keys(&m.by)?;
                m.invariant = None;
                Ok(m)
            })
            .collect::<Fallible<_>>()?,
    )?;

    let t_group_agg = Transformation::new(
        middle_domain,
        middle_metric.clone(),
        output_domain,
        middle_metric.clone(),
        Function::new(move |plan: &DslPlan| {
            #[cfg(patch_polars)]
            let output = DslPlan::GroupBy {
                input: Arc::new(plan.clone()),
                keys: stable_keys.clone(),
                predicates: vec![],
                aggs: aggs.clone(),
                apply: None,
                maintain_order,
                options: options.clone(),
            };
            #[cfg(not(patch_polars))]
            let output = DslPlan::GroupBy {
                input: Arc::new(plan.clone()),
                keys: stable_keys.clone(),
                aggs: aggs.clone(),
                apply: None,
                maintain_order,
                options: options.clone(),
            };
            output
        }),
        StabilityMap::new_fallible(move |d_in: &Bounds| {
            let contributed_rows = d_in.get_bound(&HashSet::new()).per_group;
            let contributed_groups = d_in.get_bound(&h_keys).num_groups;

            let Some(influenced_groups) = option_min(contributed_rows, contributed_groups) else {
                return fallible!(
                    FailedMap,
                    "an upper bound on the number of contributed rows or groups is required"
                );
            };

            Ok(Bounds(vec![Bound {
                by: HashSet::new(),
                per_group: Some(influenced_groups.inf_mul(&2)?),
                num_groups: None,
            }]))
        }),
    )?;

    t_prior >> t_group_agg
}

/// Express a subset of validated grouping keys as their materialized output columns.
/// Re-evaluating a grouping expression after aggregation may describe different groups.
pub(super) fn materialize_grouping_keys(by: &HashSet<Expr>) -> Fallible<HashSet<Expr>> {
    by.iter()
        .map(|key| Ok(col(key.clone().meta().output_name()?)))
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) enum Resize {
    Allow,
    Ban,
}

/// How an expression behaves when the input rows are permuted.
///
/// An invariant expression produces the same value; an equivariant expression
/// reorders its result with the input; a randomized expression has a
/// permutation-invariant distribution. Uncertified behavior is an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PermutationBehavior {
    Invariant,
    Equivariant,
    Randomized,
}

impl PermutationBehavior {
    /// Combine row-aligned arguments, preserving scalar broadcast.
    fn combine(self, rhs: Self) -> Fallible<Self> {
        use PermutationBehavior::*;
        match (self, rhs) {
            (Invariant, other) | (other, Invariant) => Ok(other),
            (Equivariant, Equivariant) => Ok(Equivariant),
            // Combining a shuffled column with an unshuffled column can
            // produce an order-dependent joint distribution.
            (Randomized, _) | (_, Randomized) => unproven(),
        }
    }

    fn reducer(self) -> Self {
        Self::Invariant
    }

    fn order_sensitive_reducer(self) -> Fallible<Self> {
        match self {
            Self::Invariant | Self::Randomized => Ok(Self::Invariant),
            Self::Equivariant => fallible!(
                MakeTransformation,
                "Order-sensitive aggregation requires a trusted random permutation of its input, or a permutation-equivariant input for an invariant reducer."
            ),
        }
    }
}

/// Fail immediately when an expression's permutation behavior cannot be proven.
fn unproven() -> Fallible<PermutationBehavior> {
    fallible!(
        MakeTransformation,
        "Expression permutation behavior is not certified for sample-and-aggregate."
    )
}

/// Validate an aggregation and prove that its result is invariant in
/// distribution under permutations of the input rows.
pub(crate) fn check_permutation_invariance(expr: &Expr, schema: &Schema) -> Fallible<()> {
    if inspect_expr(expr, Resize::Allow, schema)? == PermutationBehavior::Invariant {
        Ok(())
    } else {
        fallible!(
            MakeTransformation,
            "Aggregation expression has no established permutation-invariance proof."
        )
    }
}

/// Check that a truncation key commutes with row permutations and cannot
/// resize its input or fail depending on private values.
pub(crate) fn is_permutation_equivariant(expr: &Expr, schema: &Schema) -> bool {
    inspect_expr(expr, Resize::Ban, schema).is_ok_and(|behavior| {
        matches!(
            behavior,
            PermutationBehavior::Equivariant | PermutationBehavior::Invariant
        )
    })
}

pub(super) fn canonical_sort_dtype(dtype: &DataType) -> bool {
    match dtype {
        DataType::Float32 | DataType::Float64 => false,
        DataType::List(inner) | DataType::Array(inner, _) => canonical_sort_dtype(inner),
        DataType::Struct(fields) => fields
            .iter()
            .all(|field| canonical_sort_dtype(field.dtype())),
        _ => true,
    }
}

/// Quantile parameters are currently evaluated against the entire frame,
/// rather than separately for each group: https://github.com/pola-rs/polars/issues/20951
/// A public scalar parameter preserves group locality.
fn check_quantile(expr: &Expr) -> Fallible<()> {
    let value = match expr {
        Expr::Literal(LiteralValue::Scalar(value)) if value.dtype().is_numeric() => {
            value.value().try_extract::<f64>().ok()
        }
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Int(value))) => Some(*value as f64),
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Float(value))) => Some(*value),
        _ => None,
    };
    if value.is_some_and(|value| value.is_finite() && (0.0..=1.0).contains(&value)) {
        Ok(())
    } else {
        fallible!(
            MakeTransformation,
            "quantile requires a public numeric scalar literal in [0, 1]."
        )
    }
}

/// Return a schema-derived dtype, rather than guessing about expressions
/// whose behavior (e.g. categorical encoding) is type-dependent.
fn checked_dtype(expr: &Expr, schema: &Schema) -> Fallible<DataType> {
    Ok(expr.clone().to_field(schema)?.dtype)
}

fn simple_dtype(dtype: &DataType) -> bool {
    dtype.is_primitive_numeric() || matches!(dtype, DataType::Boolean | DataType::String)
}

/// Bound arguments must not be expressions over private rows.
fn public_numeric_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Literal(LiteralValue::Scalar(value)) if value.dtype().is_numeric() => {
            value.value().try_extract::<f64>().is_ok_and(f64::is_finite)
        }
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Int(_))) => true,
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Float(value))) => value.is_finite(),
        _ => false,
    }
}

fn public_scalar(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(value) if value.is_scalar())
}

/// Check only the small set of operations needed for ordinary bounded
/// sample-and-aggregate statistics. Everything else is unsupported.
fn inspect_expr(expr: &Expr, resize: Resize, schema: &Schema) -> Fallible<PermutationBehavior> {
    use PermutationBehavior::*;

    if let Some(data) = match_permute(expr)? {
        if matches!(resize, Resize::Ban) {
            return fallible!(
                MakeTransformation,
                "permute cannot be combined with another row-aligned expression."
            );
        }
        return match inspect_expr(data, Resize::Allow, schema)? {
            Invariant | Equivariant => Ok(Randomized),
            Randomized => unproven(),
        };
    }

    match expr {
        Expr::Element | Expr::Column(_) => Ok(Equivariant),
        Expr::Alias(expr, _) | Expr::KeepName(expr) | Expr::RenameAlias { expr, .. } => {
            inspect_expr(expr, resize, schema)
        }
        Expr::Literal(value) if value.is_scalar() => Ok(Invariant),
        Expr::Literal(_) => fallible!(
            MakeTransformation,
            "Only scalar literals are supported in grouped aggregations."
        ),
        Expr::Len => Ok(Invariant),

        Expr::BinaryExpr { .. } => unproven(),

        Expr::Cast {
            expr,
            dtype,
            options,
        } => {
            match options {
                CastOptions::NonStrict => (),
                CastOptions::Strict => {
                    return fallible!(
                        MakeTransformation,
                        "Strict casts are not supported in grouped aggregations."
                    );
                }
                CastOptions::Overflowing => {
                    return fallible!(MakeTransformation, "Overflowing casts are not supported.");
                }
            }
            let target = dtype.as_literal().ok_or_else(|| {
                err!(
                    MakeTransformation,
                    "Cast target must be a public literal dtype."
                )
            })?;
            if !target.is_primitive_numeric() && !matches!(target, DataType::Boolean) {
                return unproven();
            }
            let source = checked_dtype(expr, schema)?;
            if !public_scalar(expr) && !is_numeric_coercion(&source, target) {
                return unproven();
            }
            check_private_cast(&source, target)?;
            inspect_expr(expr, resize, schema)
        }

        Expr::Ternary {
            predicate,
            truthy,
            falsy,
        } => {
            // Polars inserts strict casts to a common supertype. Restrict
            // implicit promotions to pairs that cannot fail on private values.
            if checked_dtype(predicate, schema)? != DataType::Boolean {
                return unproven();
            }
            let truthy_dtype = checked_dtype(truthy, schema)?;
            let falsy_dtype = checked_dtype(falsy, schema)?;
            if truthy_dtype != falsy_dtype
                && truthy_dtype != DataType::Null
                && falsy_dtype != DataType::Null
                && !is_numeric_coercion(&truthy_dtype, &falsy_dtype)
            {
                return unproven();
            }
            inspect_expr(predicate, Resize::Ban, schema)?
                .combine(inspect_expr(truthy, Resize::Ban, schema)?)?
                .combine(inspect_expr(falsy, Resize::Ban, schema)?)
        }

        Expr::Filter { input, by } => {
            if matches!(resize, Resize::Ban) {
                return unproven();
            }
            if !matches!(checked_dtype(by, schema)?, DataType::Boolean) {
                return unproven();
            }
            inspect_expr(input, Resize::Ban, schema)?.combine(inspect_expr(
                by,
                Resize::Ban,
                schema,
            )?)
        }

        Expr::Agg(agg) => {
            enum Reducer {
                Invariant,
                IntegerSum,
                CanonicalMinMax,
                OrderSensitive,
            }
            use Reducer::*;

            let (input, reducer) = match agg {
                AggExpr::Count { input, .. } | AggExpr::NUnique(input) => {
                    (input.as_ref(), Invariant)
                }
                AggExpr::Sum(input) => (input.as_ref(), IntegerSum),
                AggExpr::Min { input, .. } | AggExpr::Max { input, .. } => {
                    (input.as_ref(), CanonicalMinMax)
                }
                AggExpr::First(input)
                | AggExpr::Last(input)
                | AggExpr::Mean(input)
                | AggExpr::Median(input)
                | AggExpr::Std(input, _)
                | AggExpr::Var(input, _) => (input.as_ref(), OrderSensitive),
                AggExpr::Quantile {
                    expr: input,
                    quantile,
                    ..
                } => {
                    check_quantile(quantile)?;
                    (input.as_ref(), OrderSensitive)
                }
                _ => return unproven(),
            };

            let behavior = inspect_expr(input, Resize::Allow, schema)?;
            let invariant = match reducer {
                Invariant => true,
                IntegerSum => input.clone().to_field(schema).is_ok_and(|field| {
                    field.dtype.is_integer() || matches!(field.dtype, DataType::Boolean)
                }),
                CanonicalMinMax => input
                    .clone()
                    .to_field(schema)
                    .is_ok_and(|field| canonical_sort_dtype(&field.dtype)),
                OrderSensitive => false,
            };

            if invariant {
                Ok(behavior.reducer())
            } else {
                behavior.order_sensitive_reducer()
            }
        }

        Expr::Function { input, function } => inspect_function(function, input, resize, schema),
        _ => unproven(),
    }
}

/// The function whitelist contains only simple elementwise operations and
/// aggregation primitives. A rejected function remains unsupported even if
/// its arguments happen to be permutation-equivariant.
fn inspect_function(
    function: &FunctionExpr,
    inputs: &[Expr],
    resize: Resize,
    schema: &Schema,
) -> Fallible<PermutationBehavior> {
    use PermutationBehavior::*;

    let unary = || -> Fallible<PermutationBehavior> {
        let [input] = <&[Expr; 1]>::try_from(inputs)
            .map_err(|_| err!(MakeTransformation, "Expected one function argument."))?;
        inspect_expr(input, resize, schema)
    };

    match function {
        FunctionExpr::Boolean(BooleanFunction::Any { .. })
        | FunctionExpr::Boolean(BooleanFunction::All { .. })
        | FunctionExpr::NullCount => {
            let [input] = <&[Expr; 1]>::try_from(inputs)
                .map_err(|_| err!(MakeTransformation, "Expected one aggregation argument."))?;
            inspect_expr(input, Resize::Allow, schema)?;
            Ok(Invariant)
        }
        FunctionExpr::Boolean(
            BooleanFunction::IsNull
            | BooleanFunction::IsNotNull
            | BooleanFunction::IsNan
            | BooleanFunction::IsNotNan
            | BooleanFunction::IsFinite
            | BooleanFunction::IsInfinite
            | BooleanFunction::Not,
        ) => unary(),

        // Polars negation wraps for signed integers, so it remains
        // infallible and row-wise even at the minimum representable value.
        FunctionExpr::Negate => {
            let [input] = <&[Expr; 1]>::try_from(inputs)
                .map_err(|_| err!(MakeTransformation, "Expected one numeric argument."))?;
            if !matches!(
                checked_dtype(input, schema)?,
                DataType::Int8
                    | DataType::Int16
                    | DataType::Int32
                    | DataType::Int64
                    | DataType::Float32
                    | DataType::Float64
            ) {
                return unproven();
            }
            unary()
        }

        // Float unary operations cannot overflow integer representations.
        // Negative square roots produce NaN, not an error.
        FunctionExpr::Abs
        | FunctionExpr::Floor
        | FunctionExpr::Ceil
        | FunctionExpr::Round { .. }
        | FunctionExpr::Pow(PowFunction::Sqrt) => {
            let [input] = <&[Expr; 1]>::try_from(inputs)
                .map_err(|_| err!(MakeTransformation, "Expected one numeric argument."))?;
            if !matches!(
                checked_dtype(input, schema)?,
                DataType::Float32 | DataType::Float64
            ) {
                return unproven();
            }
            unary()
        }

        FunctionExpr::AsStruct => {
            let mut behavior = Invariant;
            for input in inputs {
                behavior = behavior.combine(inspect_expr(input, Resize::Ban, schema)?)?;
            }
            Ok(behavior)
        }

        FunctionExpr::FillNull => {
            let [data, replacement] = <&[Expr; 2]>::try_from(inputs)
                .map_err(|_| err!(MakeTransformation, "fill_null requires two arguments."))?;
            if !public_scalar(replacement) {
                return unproven();
            }
            let data_dtype = checked_dtype(data, schema)?;
            let fill_dtype = checked_dtype(replacement, schema)?;
            if !simple_dtype(&data_dtype)
                || (data_dtype != fill_dtype && !is_numeric_coercion(&data_dtype, &fill_dtype))
            {
                return unproven();
            }
            inspect_expr(data, resize, schema)
        }

        FunctionExpr::Clip { has_min, has_max } => {
            let expected = 1 + usize::from(*has_min) + usize::from(*has_max);
            if inputs.len() != expected
                || !inputs.iter().skip(1).all(public_numeric_literal)
                || !checked_dtype(&inputs[0], schema)?.is_primitive_numeric()
            {
                return unproven();
            }
            inspect_expr(&inputs[0], resize, schema)
        }

        // A filtered input is allowed inside a reducer, but not wherever
        // another expression needs the original row alignment.
        FunctionExpr::DropNulls => {
            if matches!(resize, Resize::Ban) {
                return unproven();
            }
            unary()
        }

        _ => unproven(),
    }
}
