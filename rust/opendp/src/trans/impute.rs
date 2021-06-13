use std::ops::{Add, Mul, Sub};

use num::{Float, One};

use crate::core::{DatasetMetric, Domain, Function, StabilityRelation, Transformation};
use crate::dom::{AllDomain, InherentNullDomain, VectorDomain, OptionNullDomain};
use crate::error::Fallible;
use crate::dom::InherentNull;
use crate::samplers::SampleUniform;

/// A [`Transformation`] that imputes elementwise with a sample from Uniform(lower, upper).
/// Maps a Vec<T> -> Vec<T>, where the input is a type with built-in nullity.
pub fn make_impute_uniform_float<M, T>(
    lower: T, upper: T,
) -> Fallible<Transformation<VectorDomain<InherentNullDomain<AllDomain<T>>>, VectorDomain<AllDomain<T>>, M, M>>
    where M: DatasetMetric,
          for<'a> T: 'static + Float + SampleUniform + Clone + Sub<Output=T> + Mul<&'a T, Output=T> + Add<&'a T, Output=T> + InherentNull {
    if lower.is_nan() { return fallible!(MakeTransformation, "lower may not be nan"); }
    if upper.is_nan() { return fallible!(MakeTransformation, "upper may not be nan"); }
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    let scale = upper.clone() - lower.clone();

    Ok(Transformation::new(
        VectorDomain::new(InherentNullDomain::new(AllDomain::new())),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<T>| {
            arg.iter().map(|v| {
                if v.is_null() {
                    T::sample_standard_uniform(false).map(|v| v * &scale + &lower)
                } else { Ok(v.clone()) }
            }).collect()
        }),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())))
}

// utility trait to impute with a constant, regardless of the representation of null
pub trait ImputableDomain: Domain {
    type NonNull;
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::NonNull) -> &'a Self::NonNull;
    fn is_null(constant: &Self::NonNull) -> bool;
    fn new() -> Self;
}
// how to impute, when null represented as Option<T>
impl<T: Clone> ImputableDomain for OptionNullDomain<AllDomain<T>> {
    type NonNull = T;
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::NonNull) -> &'a Self::NonNull {
        default.as_ref().unwrap_or(constant)
    }
    fn is_null(_constant: &Self::NonNull) -> bool { false }
    fn new() -> Self { OptionNullDomain::new(AllDomain::new()) }
}
// how to impute, when null represented as T with internal nullity
impl<T: InherentNull> ImputableDomain for InherentNullDomain<AllDomain<T>> {
    type NonNull = Self::Carrier;
    fn impute_constant<'a>(default: &'a Self::Carrier, constant: &'a Self::NonNull) -> &'a Self::NonNull {
        if default.is_null() { constant } else { default }
    }
    fn is_null(constant: &Self::NonNull) -> bool { constant.is_null() }
    fn new() -> Self { InherentNullDomain::new(AllDomain::new()) }
}

/// A [`Transformation`] that imputes elementwise with a constant value.
/// Maps a Vec<Option<T>> -> Vec<T> if input domain is AllDomain<Option<T>>,
///     or Vec<T> -> Vec<T> if input domain is NullableDomain<AllDomain<T>>
/// Type argument DA is "Domain of the Atom"; the domain type inside VectorDomain.
pub fn make_impute_constant<DA, M>(
    constant: DA::NonNull
) -> Fallible<Transformation<VectorDomain<DA>, VectorDomain<AllDomain<DA::NonNull>>, M, M>>
    where DA: ImputableDomain,
          DA::NonNull: 'static + Clone,
          M: DatasetMetric {
    if DA::is_null(&constant) { return fallible!(MakeTransformation, "Constant may not be null.") }

    Ok(Transformation::new(
        VectorDomain::new(DA::new()),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<DA::Carrier>| arg.iter()
            .map(|v| DA::impute_constant(v, &constant))
            .cloned().collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())))
}


#[cfg(test)]
mod test {
    use crate::dist::HammingDistance;
    use crate::error::ExplainUnwrap;
    use crate::trans::{make_impute_constant, make_impute_uniform_float};
    use crate::dom::OptionNullDomain;

    #[test]
    fn test_impute_uniform() {
        let imputer = make_impute_uniform_float::<HammingDistance, f64>(2.0, 2.0).unwrap_test();

        let result = imputer.function.eval(&vec![1.0, f64::NAN]).unwrap_test();

        assert_eq!(result, vec![1., 2.]);
        assert!(imputer.stability_relation
            .eval(&1, &1).unwrap_test());
    }

    #[test]
    fn test_impute_constant() {
        let imputer = make_impute_constant::<OptionNullDomain<_>, HammingDistance>("IMPUTED".to_string()).unwrap_test();

        let result = imputer.function.eval(&vec![Some("A".to_string()), None]).unwrap_test();

        assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
        assert!(imputer.stability_relation
            .eval(&1, &1).unwrap_test());
    }
}