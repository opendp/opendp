use crate::error::*;
use crate::polars::permute::match_permute;
use crate::transformations::make_stable_expr::cast::{check_private_cast, is_numeric_coercion};
use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::plans::{DynLiteralValue, LiteralValue};

#[cfg(test)]
#[path = "permutation_test.rs"]
mod test;

#[derive(Clone, Copy)]
enum Resize {
    Allow,
    Ban,
}

/// How an expression behaves when the input rows are permuted.
///
/// Invariant scalars have a permutation-invariant distribution; equivariant
/// values reorder with the input; shuffled values have a permutation-invariant
/// distribution but are not certified as broadcastable scalars. Uncertified behavior is an error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PermutationBehavior {
    Invariant,
    Equivariant,
    Shuffled,
}

impl PermutationBehavior {
    /// Combine row-aligned arguments, preserving scalar broadcast.
    fn combine(self, rhs: Self) -> Fallible<Self> {
        use PermutationBehavior::*;
        match (self, rhs) {
            // Row-aligned combinations must not consume shuffled vectors.
            (Shuffled, _) | (_, Shuffled) => unproven(),
            (Invariant, other) | (other, Invariant) => Ok(other),
            (Equivariant, Equivariant) => Ok(Equivariant),
        }
    }

    fn order_sensitive_reducer(self) -> Fallible<Self> {
        match self {
            Self::Invariant | Self::Shuffled => Ok(Self::Invariant),
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
pub(super) fn check_permutation_invariance(expr: &Expr, schema: &Schema) -> Fallible<()> {
    if permutation_behavior(expr, Resize::Allow, Some(schema))? == PermutationBehavior::Invariant {
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
pub(super) fn is_permutation_equivariant(expr: &Expr) -> bool {
    permutation_behavior(expr, Resize::Ban, None).is_ok_and(|behavior| {
        matches!(
            behavior,
            PermutationBehavior::Equivariant | PermutationBehavior::Invariant
        )
    })
}

pub(super) fn has_canonical_representatives(dtype: &DataType) -> bool {
    !crate::polars::contains_float(dtype)
}

/// Quantile parameters are currently evaluated against the entire frame,
/// rather than separately for each group: https://github.com/pola-rs/polars/issues/20951
/// A public scalar parameter preserves group locality.
fn numeric_literal(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Literal(LiteralValue::Scalar(value)) if value.dtype().is_numeric() => {
            value.value().try_extract::<f64>().ok()
        }
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Int(value))) => Some(*value as f64),
        Expr::Literal(LiteralValue::Dyn(DynLiteralValue::Float(value))) => Some(*value),
        _ => None,
    }
}

fn check_quantile(expr: &Expr) -> Fallible<()> {
    if numeric_literal(expr).is_some_and(|value| (0.0..=1.0).contains(&value)) {
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
fn checked_dtype(expr: &Expr, schema: Option<&Schema>) -> Fallible<DataType> {
    let schema = schema.ok_or_else(|| {
        err!(
            MakeTransformation,
            "A schema is required to certify this expression."
        )
    })?;
    Ok(expr.clone().to_field(schema)?.dtype)
}

fn simple_dtype(dtype: &DataType) -> bool {
    dtype.is_primitive_numeric() || matches!(dtype, DataType::Boolean | DataType::String)
}

/// Bound arguments must not be expressions over private rows.
fn public_numeric_literal(expr: &Expr) -> bool {
    numeric_literal(expr).is_some_and(f64::is_finite)
}

fn public_scalar(expr: &Expr) -> bool {
    matches!(expr, Expr::Literal(value) if value.is_scalar())
}

/// Check only the small set of operations needed for ordinary bounded
/// sample-and-aggregate statistics. Everything else is unsupported.
fn permutation_behavior(
    expr: &Expr,
    resize: Resize,
    schema: Option<&Schema>,
) -> Fallible<PermutationBehavior> {
    use PermutationBehavior::*;

    if let Some(data) = match_permute(expr)? {
        if matches!(resize, Resize::Ban) {
            return fallible!(
                MakeTransformation,
                "permute cannot be combined with another row-aligned expression."
            );
        }
        return match permutation_behavior(data, Resize::Allow, schema)? {
            Invariant | Equivariant => Ok(Shuffled),
            Shuffled => unproven(),
        };
    }

    match expr {
        Expr::Column(_) => Ok(Equivariant),
        Expr::Alias(expr, _) | Expr::KeepName(expr) | Expr::RenameAlias { expr, .. } => {
            permutation_behavior(expr, resize, schema)
        }
        Expr::Literal(value) if value.is_scalar() => Ok(Invariant),
        Expr::Literal(_) => fallible!(
            MakeTransformation,
            "Only scalar literals are supported in grouped aggregations."
        ),
        Expr::Len => Ok(Invariant),

        // Binary expressions are not certified for infallible, permutation-
        // invariant sample-and-aggregate execution.
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
            permutation_behavior(expr, resize, schema)
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
            permutation_behavior(predicate, Resize::Ban, schema)?
                .combine(permutation_behavior(truthy, Resize::Ban, schema)?)?
                .combine(permutation_behavior(falsy, Resize::Ban, schema)?)
        }

        Expr::Filter { input, by } => {
            if matches!(resize, Resize::Ban) {
                return unproven();
            }
            if !matches!(checked_dtype(by, schema)?, DataType::Boolean) {
                return unproven();
            }
            permutation_behavior(input, Resize::Ban, schema)?.combine(permutation_behavior(
                by,
                Resize::Ban,
                schema,
            )?)
        }

        Expr::Agg(agg) => {
            let (input, invariant) = match agg {
                AggExpr::Count { input, .. } | AggExpr::NUnique(input) => (input, true),
                AggExpr::Sum(input) => (
                    input,
                    checked_dtype(input, schema).is_ok_and(|dtype| {
                        dtype.is_integer() || matches!(dtype, DataType::Boolean)
                    }),
                ),
                AggExpr::Min { input, .. } | AggExpr::Max { input, .. } => (
                    input,
                    checked_dtype(input, schema)
                        .is_ok_and(|dtype| has_canonical_representatives(&dtype)),
                ),
                AggExpr::First(input)
                | AggExpr::Last(input)
                | AggExpr::Mean(input)
                | AggExpr::Median(input)
                | AggExpr::Std(input, _)
                | AggExpr::Var(input, _) => (input, false),
                AggExpr::Quantile {
                    expr: input,
                    quantile,
                    ..
                } => {
                    check_quantile(quantile)?;
                    (input, false)
                }
                _ => return unproven(),
            };

            let behavior = permutation_behavior(input, Resize::Allow, schema)?;
            if invariant {
                Ok(Invariant)
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
    schema: Option<&Schema>,
) -> Fallible<PermutationBehavior> {
    use PermutationBehavior::*;

    let unary_input = || -> Fallible<&Expr> {
        let [input] = inputs else {
            return fallible!(MakeTransformation, "Expected one function argument.");
        };
        Ok(input)
    };
    let unary = || permutation_behavior(unary_input()?, resize, schema);

    match function {
        FunctionExpr::Boolean(BooleanFunction::Any { .. })
        | FunctionExpr::Boolean(BooleanFunction::All { .. })
        | FunctionExpr::NullCount => {
            let input = unary_input()?;
            permutation_behavior(input, Resize::Allow, schema)?;
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

        // Float unary operations are row-local and cannot overflow integer
        // representations. Negative square roots produce NaN, not an error.
        FunctionExpr::Abs
        | FunctionExpr::Negate
        | FunctionExpr::Floor
        | FunctionExpr::Ceil
        | FunctionExpr::Round { .. }
        | FunctionExpr::Pow(PowFunction::Sqrt) => {
            let input = unary_input()?;
            if !matches!(
                checked_dtype(input, schema)?,
                DataType::Float32 | DataType::Float64
            ) {
                return unproven();
            }
            permutation_behavior(input, resize, schema)
        }

        FunctionExpr::AsStruct => {
            let mut behavior = Invariant;
            for input in inputs {
                behavior = behavior.combine(permutation_behavior(input, Resize::Ban, schema)?)?;
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
            permutation_behavior(data, resize, schema)
        }

        FunctionExpr::Clip { has_min, has_max } => {
            let expected = 1 + usize::from(*has_min) + usize::from(*has_max);
            if inputs.len() != expected
                || !inputs.iter().skip(1).all(public_numeric_literal)
                || !checked_dtype(&inputs[0], schema)?.is_primitive_numeric()
            {
                return unproven();
            }
            permutation_behavior(&inputs[0], resize, schema)
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
