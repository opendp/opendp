#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::EventLevelMetric;
use crate::traits::{CheckAtom, HasNull, RoundCast};
use crate::transformations::make_row_by_row;

#[bootstrap(features("contrib"), generics(M(suppress), TIA(suppress)))]
/// Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
/// For each element, failure to parse results in `None`, else `Some(out)`.
///
/// Can be chained with `make_impute_constant` or `make_drop_null` to handle nullity.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast<M, TIA, TOA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
) -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        M,
        VectorDomain<OptionDomain<AtomDomain<TOA>>>,
        M,
    >,
>
where
    M: EventLevelMetric,
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + CheckAtom,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<OptionDomain<AtomDomain<TOA>>>, M): MetricSpace,
{
    make_row_by_row(
        input_domain,
        input_metric,
        OptionDomain::new(AtomDomain::default()),
        |v| {
            TOA::round_cast(v.clone())
                .ok()
                .and_then(|v| if v.is_null() { None } else { Some(v) })
        },
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        input_domain(c_type = "AnyDomain *"),
        input_metric(c_type = "AnyMetric *")
    ),
    generics(TIA(suppress), M(suppress)),
    derived_types(
        TIA = "$get_atom(get_type(input_domain))",
        M = "$get_type(input_metric)"
    )
)]
/// Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
/// Any element that fails to cast is filled with default.
///
///
/// | `TIA`  | `TIA::default()` |
/// | ------ | ---------------- |
/// | float  | `0.`             |
/// | int    | `0`              |
/// | string | `""`             |
/// | bool   | `false`          |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_default<M, TIA, TOA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, M, VectorDomain<AtomDomain<TOA>>, M>>
where
    M: EventLevelMetric,
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + Default + CheckAtom,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    make_row_by_row(input_domain, input_metric, AtomDomain::default(), |v| {
        TOA::round_cast(v.clone()).unwrap_or_default()
    })
}

#[bootstrap(features("contrib"), generics(M(suppress), TIA(suppress)))]
/// Make a Transformation that casts a vector of data from type `TIA` to a type that can represent nullity `TOA`.
/// If cast fails, fill with `TOA`'s null value.
///
/// | `TIA`  | `TIA::default()` |
/// | ------ | ---------------- |
/// | float  | NaN              |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_inherent<M, TIA, TOA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, M, VectorDomain<AtomDomain<TOA>>, M>>
where
    M: EventLevelMetric,
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + HasNull + CheckAtom,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    make_row_by_row(input_domain, input_metric, AtomDomain::default(), |v| {
        TOA::round_cast(v.clone()).unwrap_or(TOA::NULL)
    })
}

#[cfg(test)]
mod test;

#[bootstrap(features("contrib"), generics(M(suppress), TIA(suppress)))]
/// Make a Transformation that casts a measurement output to int 64.
/// Casting measurements to i64 before noise is added can enable negative values.
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
/// * `expr` - The input measurement to be cast
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_measurement_to_i64<MI, const P: usize>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: M,
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
                "make_cast_measurement_to_i64 only supports literal dtype"
            )
        })?
        .clone();

    if to_type_dtype != DataType::Int64 {
        return fallible!(
            MakeTransformation,
            "make_cast_measurement_to_i64 cast expects target dtype Int64, found {}",
            to_type_dtype
        );
    }

    if matches!(options, CastOptions::Strict) {
        options = CastOptions::NonStrict;
    }

    // This recursively stabilizes len/count/n_unique/count_null.
    // The child emits ExprDomain under LpDistance<P, f64>.
    let t_prior = input
        .as_ref()
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;

    let (middle_domain, middle_metric) = t_prior.output_space();

    let mut output_domain = middle_domain.clone();
    let active_series = &mut output_domain.column;

    active_series.set_element_domain(AtomDomain::<i64>::default());

    t_prior
        >> Transformation::new(
            middle_domain.clone(),
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
