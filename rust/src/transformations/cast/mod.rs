#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{MetricSpace, Transformation};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{CheckAtom, InherentNull, RoundCast};
use crate::transformations::make_row_by_row;

use super::DatasetMetric;

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
        VectorDomain<OptionDomain<AtomDomain<TOA>>>,
        M,
        M,
    >,
>
where
    M: DatasetMetric,
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
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, VectorDomain<AtomDomain<TOA>>, M, M>>
where
    M: DatasetMetric,
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
) -> Fallible<Transformation<VectorDomain<AtomDomain<TIA>>, VectorDomain<AtomDomain<TOA>>, M, M>>
where
    M: DatasetMetric,
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + InherentNull + CheckAtom,
    (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    (VectorDomain<AtomDomain<TOA>>, M): MetricSpace,
{
    make_row_by_row(
        input_domain,
        input_metric,
        AtomDomain::new_nullable(),
        |v| TOA::round_cast(v.clone()).unwrap_or(TOA::NULL),
    )
}

#[cfg(test)]
mod test;
