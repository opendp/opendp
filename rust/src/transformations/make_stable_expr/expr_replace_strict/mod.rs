use polars::prelude::*;
use polars_plan::dsl::Expr;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{CategoricalDomain, ExprDomain, OuterMetric, WildExprDomain};
use crate::error::*;
use crate::metrics::MicrodataMetric;
use crate::polars::ExtractValue;
use crate::transformations::expr_replace::materialize_lit_list;

use super::StableExpr;
use super::expr_replace::{is_cast_fallible, literal_is_nullable, literal_len};

#[cfg(test)]
mod test;

/// Make a Transformation that returns a `replace_strict(old, new, default)` expression for a LazyFrame.
///
/// # Arguments
/// * `input_domain` - Expr domain
/// * `input_metric` - The metric under which neighboring LazyFrames are compared
/// * `expr` - The replace expression
pub fn make_expr_replace_strict<M: OuterMetric>(
    input_domain: WildExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, M, ExprDomain, M>>
where
    M::InnerMetric: MicrodataMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
    Expr: StableExpr<M, M>,
{
    let Expr::Function {
        mut input,
        function: FunctionExpr::ReplaceStrict { return_dtype },
    } = expr
    else {
        return fallible!(MakeTransformation, "expected replace_strict expression");
    };

    let return_dtype_lit = match return_dtype.as_ref() {
        Some(expr) => Some(
            expr.as_literal()
                .ok_or_else(|| err!(MakeTransformation, "return_dtype must be literal"))?,
        ),
        None => None,
    };

    if input.len() == 3 {
        input.push(Expr::Literal(LiteralValue::untyped_null()));
    }

    // check arguments
    let [input, old, new, default] = <[Expr; 4]>::try_from(input).map_err(|_| {
        err!(
            MakeTransformation,
            "replace_strict takes four arguments: input, old, new and default"
        )
    })?;

    let t_prior = input.make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let (Expr::Literal(old_lit), Expr::Literal(new_lit), Expr::Literal(default_lit)) =
        (&old, &new, &default)
    else {
        return fallible!(
            MakeTransformation,
            "replace_strict: old, new and default must be literals, but found {:?}, {:?} and {:?}",
            old,
            new,
            default
        );
    };

    let old_lit = materialize_lit_list(old_lit)?;
    let new_lit = materialize_lit_list(new_lit)?;

    // check lengths
    let (old_len, new_len) = (literal_len(&old_lit)?, literal_len(&new_lit)?);
    if ![old_len, 1].contains(&new_len) {
        return fallible!(
            MakeTransformation,
            "length of `new` ({}) must match length of `old` ({}) or be broadcastable (1)",
            new_len,
            old_len
        );
    }
    if literal_len(default_lit)? != 1 {
        return fallible!(
            MakeTransformation,
            "length of `default` ({}) must be 1",
            literal_len(default_lit)?
        );
    };

    // check data types
    let input_dtype = middle_domain.column.dtype();
    if matches!(input_dtype, DataType::Categorical(_, _)) {
        let categories = middle_domain
            .column
            .element_domain::<CategoricalDomain>()?
            .categories();

        // allow replacement on categoricals when old lit matches categories
        match (Series::extract(old_lit.clone()), categories) {
            (Ok(Some(old_lit)), Some(categories))
                if old_lit.cast(&DataType::String)?
                    == Series::new(
                        old_lit.name().clone(),
                        categories
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>(),
                    ) =>
            {
                ()
            }
            _ => {
                return fallible!(
                    MakeTransformation,
                    "replace_strict cannot be applied to categorical data ({}), because it may trigger a data-dependent CategoricalRemappingWarning in Polars",
                    middle_domain.column.name
                );
            }
        }
    }

    let old_dtype = old_lit.get_datatype();
    if is_cast_fallible(&old_dtype, &input_dtype) {
        return fallible!(
            MakeTransformation,
            "replace_strict: old datatype ({:?}) must be consistent with the input datatype ({:?})",
            old_dtype,
            input_dtype
        );
    }

    let new_dtype = new_lit.get_datatype();
    if let Some(return_dtype_lit) = return_dtype_lit {
        if is_cast_fallible(&new_dtype, return_dtype_lit) {
            return fallible!(
                MakeTransformation,
                "replace_strict: new datatype ({}) must be consistent with the return datatype ({})",
                new_dtype,
                return_dtype_lit
            );
        }
    }

    let default_dtype = default_lit.get_datatype();
    if is_cast_fallible(&default_dtype, &new_dtype) {
        return fallible!(
            MakeTransformation,
            "replace_strict: default datatype ({}) must be consistent with the new datatype ({})",
            default_dtype,
            new_dtype
        );
    }

    let mut output_domain = middle_domain.clone();

    // reset domain descriptors
    output_domain.column.set_dtype(new_dtype)?;

    // if replacement can introduce nulls, then set nullable
    output_domain.column.nullable =
        literal_is_nullable(&new_lit) || literal_is_nullable(&default_lit);

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Function {
                input: vec![expr.clone(), old.clone(), new.clone(), default.clone()],
                function: FunctionExpr::ReplaceStrict {
                    return_dtype: return_dtype.clone(),
                },
            }),
            StabilityMap::new(Clone::clone),
        )?
}
