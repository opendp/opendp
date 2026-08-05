use std::{fmt::Debug, sync::Arc};

use polars::chunked_array::cast::CastOptions;
use polars::prelude::*;
use polars_plan::dsl::Expr;

use super::StableExpr;
use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, ExprDomain, SeriesDomain, WildExprDomain};
use crate::error::*;
use crate::metrics::{L01InfDistance, LpDistance};
use crate::traits::{CheckAtom, ExactIntCast, FiniteBounds};
use crate::transformations::traits::UnboundedMetric;

#[cfg(test)]
mod test;

/// Make a Transformation that casts an integer aggregation output to a signed integer type.
///
/// A cast is accepted when the input domain, including any bounds that are tighter than the
/// input type's natural bounds, is representable by the output type.
pub fn make_cast_aggregation<MI, const P: usize>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, L01InfDistance<MI>, ExprDomain, LpDistance<P, f64>>>
where
    MI: 'static + UnboundedMetric,
    (WildExprDomain, L01InfDistance<MI>): MetricSpace,
    (ExprDomain, LpDistance<P, f64>): MetricSpace,
    Expr: StableExpr<L01InfDistance<MI>, LpDistance<P, f64>>,
{
    let Expr::Cast {
        expr: input,
        dtype: to_type,
        mut options,
    } = expr
    else {
        return fallible!(MakeTransformation, "expected cast expression");
    };

    let to_type_dtype = to_type
        .as_literal()
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "make_cast_aggregation only supports literal dtype"
            )
        })?
        .clone();

    // Errors can reveal private aggregate values, so always use a non-strict cast.
    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain, input_metric)?;
    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let source_dtype = middle_domain.column.dtype();
    let cast_succeeded = set_cast_domain(
        &middle_domain.column,
        &mut output_domain.column,
        &to_type_dtype,
    );

    if !cast_succeeded {
        return fallible!(
            MakeTransformation,
            "cannot downcast from {} to {}: input domain bounds are not representable by the target type",
            source_dtype,
            to_type_dtype
        );
    }

    t_prior
        >> Transformation::new(
            middle_domain,
            middle_metric.clone(),
            output_domain,
            middle_metric,
            Function::then_expr(move |expr| Expr::Cast {
                expr: Arc::new(expr),
                dtype: to_type.clone(),
                options,
            }),
            StabilityMap::new(Clone::clone),
        )?
}

fn set_cast_domain(
    source: &SeriesDomain,
    target: &mut SeriesDomain,
    target_dtype: &DataType,
) -> bool {
    macro_rules! cast_from {
        ($source_ty:ty) => {
            match target_dtype {
                DataType::Int8 => cast_atom_domain::<$source_ty, i8>(source)
                    .map(|domain| target.set_element_domain(domain)),
                DataType::Int16 => cast_atom_domain::<$source_ty, i16>(source)
                    .map(|domain| target.set_element_domain(domain)),
                DataType::Int32 => cast_atom_domain::<$source_ty, i32>(source)
                    .map(|domain| target.set_element_domain(domain)),
                DataType::Int64 => cast_atom_domain::<$source_ty, i64>(source)
                    .map(|domain| target.set_element_domain(domain)),
                _ => None,
            }
        };
    }

    match source.dtype() {
        DataType::UInt32 => cast_from!(u32),
        DataType::UInt64 => cast_from!(u64),
        DataType::Int8 => cast_from!(i8),
        DataType::Int16 => cast_from!(i16),
        DataType::Int32 => cast_from!(i32),
        DataType::Int64 => cast_from!(i64),
        _ => None,
    }
    .is_some()
}

fn cast_atom_domain<TI, TO>(source: &SeriesDomain) -> Option<AtomDomain<TO>>
where
    TI: 'static + Clone + CheckAtom + FiniteBounds,
    TO: 'static + CheckAtom + ExactIntCast<TI> + PartialOrd + Debug,
{
    let source = source.atom_domain::<TI>().ok()?;

    let Some(bounds) = &source.bounds else {
        TO::exact_int_cast(TI::MIN_FINITE).ok()?;
        TO::exact_int_cast(TI::MAX_FINITE).ok()?;
        return Some(AtomDomain::default());
    };

    let (lower, upper) = bounds.get_closed().ok()?;
    AtomDomain::new_closed((
        TO::exact_int_cast(lower).ok()?,
        TO::exact_int_cast(upper).ok()?,
    ))
    .ok()
}
