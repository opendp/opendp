#[cfg(feature="ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::{Domain, Transformation, Function, StabilityMap};
use crate::domains::{AllDomain, InherentNullDomain, VectorDomain, OptionNullDomain};
use crate::error::Fallible;
use crate::traits::samplers::SampleUniform;
use crate::transformations::{make_row_by_row, make_row_by_row_fallible};
use crate::metrics::SymmetricDistance;
use crate::traits::{CheckNull, InherentNull, Float};

#[bootstrap(
    features("contrib"), 
    generics(TA(example = "$get_first(bounds)"))
)]
/// Make a Transformation that replaces NaN values in `Vec<TA>` with uniformly distributed floats within `bounds`.
/// 
/// # Arguments
/// * `bounds` - Tuple of inclusive lower and upper bounds.
/// 
/// # Generics
/// * `TA` - Atomic Type of data being imputed. One of `f32` or `f64`
pub fn make_impute_uniform_float<TA>(
    bounds: (TA, TA)
) -> Fallible<Transformation<VectorDomain<InherentNullDomain<AllDomain<TA>>>, VectorDomain<AllDomain<TA>>, SymmetricDistance, SymmetricDistance>>
    where TA: Float + SampleUniform {
    let (lower, upper) = bounds;
    if lower.is_nan() { return fallible!(MakeTransformation, "lower may not be nan"); }
    if upper.is_nan() { return fallible!(MakeTransformation, "upper may not be nan"); }
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    let scale = upper - lower;

    make_row_by_row_fallible(
        InherentNullDomain::new(AllDomain::new()),
        AllDomain::new(),
        move |v| if v.is_null() {
            TA::sample_standard_uniform(false).map(|v| v * scale + lower)
        } else { Ok(*v) })
}

/// Utility trait to impute with a constant, regardless of the representation of nullity.
pub trait ImputeConstantDomain: Domain {
    /// This is the type of `Self::Carrier` after imputation.
    /// 
    /// On any type `D` for which the `ImputeConstantDomain` trait is implemented, 
    /// the syntax `D::Imputed` refers to this associated type.
    /// For example, consider `D` to be `OptionNullDomain<T>`, the domain of all `Option<T>`.
    /// The implementation of this trait for `OptionNullDomain<T>` designates that `type Imputed = T`. 
    /// Thus `OptionNullDomain<T>::Imputed` is `T`.
    /// 
    /// # Proof Definition
    /// `Self::Imputed` can represent the set of possible output values after imputation.
    type Imputed;

    /// A function that replaces a potentially-null carrier type with a non-null imputed type.
    /// 
    /// # Proof Definition
    /// For any setting of the input parameters, where `constant` is non-null,
    /// the function returns a non-null value.
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::Imputed) -> &'a Self::Imputed;
    
}
// how to impute, when null represented as Option<T>
impl<T: 'static + CheckNull> ImputeConstantDomain for OptionNullDomain<AllDomain<T>> {
    type Imputed = T;
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::Imputed) -> &'a Self::Imputed {
        default.as_ref().unwrap_or(constant)
    }
}
// how to impute, when null represented as T with internal nullity
impl<T: InherentNull> ImputeConstantDomain for InherentNullDomain<AllDomain<T>> {
    type Imputed = Self::Carrier;
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::Imputed) -> &'a Self::Imputed {
        if default.is_null() { constant } else { default }
    }
}

#[bootstrap(
    features("contrib"), 
    arguments(constant(rust_type = "TA", c_type="AnyObject *")),
    generics(DIA(default = "OptionNullDomain<AllDomain<TA>>", generics = "TA")),
    derived_types(TA = "$get_atom_or_infer(DIA, constant)")
)]
/// Make a Transformation that replaces null/None data with `constant`.
/// 
/// By default, the input type is `Vec<Option<TA>>`, as emitted by make_cast.
/// Set `DA` to `InherentNullDomain<AllDomain<TA>>` for imputing on types 
/// that have an inherent representation of nullity, like floats.
/// 
/// | Atom Input Domain `DIA`             |  Input Type       | `DIA::Imputed` |
/// | ----------------------------------- | ----------------- | -------------- |
/// | `OptionNullDomain<AllDomain<TA>>`   | `Vec<Option<TA>>` | `TA`           |
/// | `InherentNullDomain<AllDomain<TA>>` | `Vec<TA>`         | `TA`           |
/// 
/// # Arguments
/// * `constant` - Value to replace nulls with.
/// 
/// # Generics
/// * `DIA` - Atomic Input Domain of data being imputed.
pub fn make_impute_constant<DIA>(
    constant: DIA::Imputed
) -> Fallible<Transformation<VectorDomain<DIA>, VectorDomain<AllDomain<DIA::Imputed>>, SymmetricDistance, SymmetricDistance>>
    where DIA: ImputeConstantDomain + Default,
          DIA::Imputed: 'static + Clone + CheckNull,
          DIA::Carrier: 'static {
    if constant.is_null() { return fallible!(MakeTransformation, "Constant may not be null.") }

    make_row_by_row(
        DIA::default(),
        AllDomain::new(),
        move |v| DIA::impute_constant(v, &constant).clone())
}

/// Utility trait to drop null values from a dataset, regardless of the representation of nullity.
pub trait DropNullDomain: Domain {
    /// This is the type of `Self::Carrier` after dropping null.
    /// 
    /// On any type `D` for which the `DropNullDomain` trait is implemented, 
    /// the syntax `D::Imputed` refers to this associated type.
    /// For example, consider `D` to be `OptionNullDomain<T>`, the domain of all `Option<T>`.
    /// The implementation of this trait for `DropNullDomain<T>` designates that `type Imputed = T`. 
    /// Thus `DropNullDomain<T>::Imputed` is `T`.
    type Imputed: 'static;

    /// Standardizes `D::Carrier` into an `Option<D::Imputed>`, where `D::Imputed` is never null.
    /// 
    /// `Self::Imputed` may have the capacity to represent null (like `f64`), 
    /// but implementations of this function must guarantee that `Self::Imputed` is never null.
    fn option(value: &Self::Carrier) -> Option<Self::Imputed>;
}

/// how to standardize into an option, when null represented as Option<T>
impl<T: 'static + CheckNull + Clone> DropNullDomain for OptionNullDomain<AllDomain<T>> {
    type Imputed = T;
    fn option(value: &Self::Carrier) -> Option<T> {
        if value.is_null() { None } else { value.clone() }
    }
}
/// how to standardize into an option, when null represented as T with internal nullity
impl<T: InherentNull + Clone> DropNullDomain for InherentNullDomain<AllDomain<T>> {
    type Imputed = T;
    fn option(value: &Self::Carrier) -> Option<T> {
        if value.is_null() { None } else { Some(value.clone()) }
    }
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that drops null values.
/// 
/// 
/// | `DA`                                | `DA::Imputed` |
/// | ----------------------------------- | ------------- |
/// | `OptionNullDomain<AllDomain<TA>>`   | `TA`          |
/// | `InherentNullDomain<AllDomain<TA>>` | `TA`          |
/// 
/// # Generics
/// * `DA` - atomic domain of input data that contains nulls.
pub fn make_drop_null<DA>(
) -> Fallible<Transformation<VectorDomain<DA>, VectorDomain<AllDomain<DA::Imputed>>, SymmetricDistance, SymmetricDistance>>
    where DA: DropNullDomain + Default, 
          DA::Imputed: CheckNull {
    Ok(Transformation::new(
        VectorDomain::new(DA::default()),
        VectorDomain::new_all(),
        Function::new(|arg: &Vec<DA::Carrier>|
            arg.iter().filter_map(DA::option).collect()),
        SymmetricDistance::default(),
        SymmetricDistance::default(),
        StabilityMap::new_from_constant(1)
    ))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impute_uniform() -> Fallible<()> {
        let imputer = make_impute_uniform_float::<f64>((2.0, 2.0))?;

        let result = imputer.invoke(&vec![1.0, f64::NAN])?;

        assert_eq!(result, vec![1., 2.]);
        assert!(imputer.check(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_impute_constant_option() -> Fallible<()> {
        let imputer = make_impute_constant::<OptionNullDomain<_>>("IMPUTED".to_string())?;

        let result = imputer.invoke(&vec![Some("A".to_string()), None])?;

        assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
        assert!(imputer.check(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_impute_constant_inherent() -> Fallible<()> {
        let imputer = make_impute_constant::<InherentNullDomain<_>>(12.)?;

        let result = imputer.invoke(&vec![f64::NAN, 23.])?;

        assert_eq!(result, vec![12., 23.]);
        assert!(imputer.check(&1, &1)?);
        Ok(())
    }

    #[test]
    fn test_impute_drop_option() -> Fallible<()> {
        let imputer = make_drop_null::<OptionNullDomain<_>>()?;

        let result = imputer.invoke(&vec![Some(f64::NAN), Some(23.), None])?;

        assert_eq!(result, vec![23.]);
        assert!(imputer.check(&1, &1)?);
        Ok(())
    }
    #[test]
    fn test_impute_drop_inherent() -> Fallible<()> {
        let imputer = make_drop_null::<InherentNullDomain<_>>()?;

        let result = imputer.invoke(&vec![f64::NAN, 23.])?;

        assert_eq!(result, vec![23.]);
        assert!(imputer.check(&1, &1)?);
        Ok(())
    }
}