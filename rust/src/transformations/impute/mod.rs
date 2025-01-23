#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{Domain, Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::samplers::GeneratorOpenDP;
use crate::traits::{CheckAtom, CheckNull, Float, InherentNull};
use crate::transformations::make_row_by_row;

use super::DatasetMetric;
use rand::distributions::{uniform::SampleUniform, Distribution, Uniform};

#[bootstrap(
    features("contrib"),
    generics(M(suppress), TA(suppress)),
    derived_types(TA = "$get_atom(get_type(input_domain))")
)]
/// Make a Transformation that replaces NaN values in `Vec<TA>` with uniformly distributed floats within `bounds`.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `bounds` - Tuple of inclusive lower and upper bounds.
///
/// # Generics
/// * `M` - Metric Type. A dataset metric.
/// * `TA` - Atomic Type of data being imputed. One of `f32` or `f64`
pub fn make_impute_uniform_float<M, TA>(
    input_domain: VectorDomain<AtomDomain<TA>>,
    input_metric: M,
    bounds: (TA, TA),
) -> Fallible<Transformation<VectorDomain<AtomDomain<TA>>, VectorDomain<AtomDomain<TA>>, M, M>>
where
    TA: Float + SampleUniform,
    M: DatasetMetric,
    (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
{
    let (lower, upper) = bounds;
    if lower.is_nan() {
        return fallible!(MakeTransformation, "lower may not be nan");
    }
    if upper.is_nan() {
        return fallible!(MakeTransformation, "upper may not be nan");
    }
    if lower >= upper {
        return fallible!(MakeTransformation, "lower must be smaller than upper");
    }

    make_row_by_row(
        input_domain,
        input_metric,
        AtomDomain::default(),
        move |v| {
            if v.is_null() {
                let mut rng = GeneratorOpenDP::new();
                let sample = Uniform::from(lower..upper).sample(&mut rng);
                rng.error.map(|_| sample).unwrap_or(lower)
            } else {
                *v
            }
        },
    )
}

/// Utility trait to impute with a constant, regardless of the representation of nullity.
pub trait ImputeConstantDomain: Domain {
    /// This is the type of `Self::Carrier` after imputation.
    ///
    /// On any type `D` for which the `ImputeConstantDomain` trait is implemented,
    /// the syntax `D::Imputed` refers to this associated type.
    /// For example, consider `D` to be `OptionDomain<T>`, the domain of all `Option<T>`.
    /// The implementation of this trait for `OptionDomain<T>` designates that `type Imputed = T`.
    /// Thus `OptionDomain<T>::Imputed` is `T`.
    ///
    /// # Proof Definition
    /// `Self::Imputed` can represent the set of possible output values after imputation.
    type Imputed;

    /// A function that replaces a potentially-null carrier type with a non-null imputed type.
    ///
    /// # Proof Definition
    /// For any setting of the input parameters, where `constant` is non-null,
    /// the function returns a non-null value.
    fn impute_constant<'a>(
        default: &'a Self::Carrier,
        constant: &'a Self::Imputed,
    ) -> &'a Self::Imputed;
}
// how to impute, when null represented as `Option<T>`
impl<T: CheckAtom> ImputeConstantDomain for OptionDomain<AtomDomain<T>> {
    type Imputed = T;
    fn impute_constant<'a>(
        default: &'a Self::Carrier,
        constant: &'a Self::Imputed,
    ) -> &'a Self::Imputed {
        default.as_ref().unwrap_or(constant)
    }
}
// how to impute, when null represented as T with internal nullity
impl<T: CheckAtom + InherentNull> ImputeConstantDomain for AtomDomain<T> {
    type Imputed = Self::Carrier;
    fn impute_constant<'a>(
        default: &'a Self::Carrier,
        constant: &'a Self::Imputed,
    ) -> &'a Self::Imputed {
        if default.is_null() {
            constant
        } else {
            default
        }
    }
}

#[bootstrap(
    features("contrib"),
    arguments(constant(
        rust_type = "$get_atom(get_type(input_domain))",
        c_type = "AnyObject *"
    )),
    generics(DIA(suppress), M(suppress))
)]
/// Make a Transformation that replaces null/None data with `constant`.
///
/// If chaining after a `make_cast`, the input type is `Option<Vec<TA>>`.
/// If chaining after a `make_cast_inherent`, the input type is `Vec<TA>`, where `TA` may take on float NaNs.
///
/// | input_domain                                    |  Input Data Type  |
/// | ----------------------------------------------- | ----------------- |
/// | `vector_domain(option_domain(atom_domain(TA)))` | `Vec<Option<TA>>` |
/// | `vector_domain(atom_domain(TA))`                | `Vec<TA>`         |
///
/// # Arguments
/// * `input_domain` - Domain of the input data. See table above.
/// * `input_metric` - Metric of the input data. A dataset metric.
/// * `constant` - Value to replace nulls with.
///
/// # Generics
/// * `DIA` - Atomic Input Domain of data being imputed.
/// * `M` - Dataset Metric.
pub fn make_impute_constant<DIA, M>(
    input_domain: VectorDomain<DIA>,
    input_metric: M,
    constant: DIA::Imputed,
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<AtomDomain<DIA::Imputed>>, M, M>>
where
    DIA: ImputeConstantDomain + Default,
    DIA::Imputed: 'static + Clone + CheckAtom,
    DIA::Carrier: 'static,
    M: DatasetMetric,
    (VectorDomain<DIA>, M): MetricSpace,
    (VectorDomain<AtomDomain<DIA::Imputed>>, M): MetricSpace,
{
    let output_atom_domain = AtomDomain::default();
    if !output_atom_domain.member(&constant)? {
        return fallible!(MakeTransformation, "Constant may not be null.");
    }

    make_row_by_row(input_domain, input_metric, output_atom_domain, move |v| {
        DIA::impute_constant(v, &constant).clone()
    })
}

/// Utility trait to drop null values from a dataset, regardless of the representation of nullity.
pub trait DropNullDomain: Domain {
    /// This is the type of `Self::Carrier` after dropping null.
    ///
    /// On any type `D` for which the `DropNullDomain` trait is implemented,
    /// the syntax `D::Imputed` refers to this associated type.
    /// For example, consider `D` to be `OptionDomain<T>`, the domain of all `Option<T>`.
    /// The implementation of this trait for `DropNullDomain<T>` designates that `type Imputed = T`.
    /// Thus `DropNullDomain<T>::Imputed` is `T`.
    type Imputed;

    /// Standardizes `D::Carrier` into an `Option<D::Imputed>`, where `D::Imputed` is never null.
    ///
    /// `Self::Imputed` may have the capacity to represent null (like `f64`),
    /// but implementations of this function must guarantee that `Self::Imputed` is never null.
    fn option(value: &Self::Carrier) -> Option<Self::Imputed>;
}

/// how to standardize into an option, when null represented as `Option<T>`
impl<T: CheckAtom + Clone> DropNullDomain for OptionDomain<AtomDomain<T>> {
    type Imputed = T;
    fn option(value: &Self::Carrier) -> Option<T> {
        if value.is_null() {
            None
        } else {
            value.clone()
        }
    }
}
/// how to standardize into an option, when null represented as T with internal nullity
impl<T: CheckAtom + InherentNull + Clone> DropNullDomain for AtomDomain<T> {
    type Imputed = T;
    fn option(value: &Self::Carrier) -> Option<T> {
        if value.is_null() {
            None
        } else {
            Some(value.clone())
        }
    }
}

#[bootstrap(features("contrib"), generics(DIA(suppress), M(suppress)))]
/// Make a Transformation that drops null values.
///
///
/// | input_domain                                    |
/// | ----------------------------------------------- |
/// | `vector_domain(option_domain(atom_domain(TA)))` |
/// | `vector_domain(atom_domain(TA))`                |
///
/// # Arguments
/// * `input_domain` - Domain of input data
/// * `input_metric` - Metric on input domain
///
/// # Generics
/// * `M` - Dataset Metric.
/// * `DIA` - atomic domain of input data that contains nulls.
pub fn make_drop_null<M, DIA>(
    input_domain: VectorDomain<DIA>,
    input_metric: M,
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<AtomDomain<DIA::Imputed>>, M, M>>
where
    DIA: DropNullDomain + Default,
    DIA::Imputed: CheckAtom,
    M: DatasetMetric,
    (VectorDomain<DIA>, M): MetricSpace,
    (VectorDomain<AtomDomain<DIA::Imputed>>, M): MetricSpace,
{
    Transformation::new(
        input_domain,
        VectorDomain::new(AtomDomain::default()),
        Function::new(|arg: &Vec<DIA::Carrier>| arg.iter().filter_map(DIA::option).collect()),
        input_metric.clone(),
        input_metric,
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod test;
