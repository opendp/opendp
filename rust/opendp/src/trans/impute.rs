use std::ops::{Add, Mul, Sub};

use num::{Float, One};

use crate::core::{DatasetMetric, Function, StabilityRelation, Transformation, Domain};
use crate::dom::{AllDomain, VectorDomain, NullableDomain};
use crate::error::Fallible;
use crate::samplers::SampleUniform;
use crate::traits::DistanceConstant;

/// A [`Transformation`] that imputes elementwise with a sample from Uniform(lower, upper).
/// Maps a Vec<T> -> Vec<T>, where the input is a type with built-in nullity.
pub fn make_impute_uniform_float<M, T>(
    lower: T, upper: T,
) -> Fallible<Transformation<VectorDomain<NullableDomain<AllDomain<T>>>, VectorDomain<AllDomain<T>>, M, M>>
    where M: DatasetMetric<Distance=u32>,
          for<'a> T: 'static + Float + SampleUniform + Clone + Sub<Output=T> + Mul<&'a T, Output=T> + Add<&'a T, Output=T>,
          M::Distance: One + DistanceConstant {
    if lower.is_nan() { return fallible!(MakeTransformation, "lower may not be nan"); }
    if upper.is_nan() { return fallible!(MakeTransformation, "upper may not be nan"); }
    if lower > upper { return fallible!(MakeTransformation, "lower may not be greater than upper") }
    let scale = upper.clone() - lower.clone();

    Ok(Transformation::new(
        VectorDomain::new(NullableDomain::new(AllDomain::new())),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<T>| {
            arg.iter().map(|v| {
                if v.is_nan() {
                    T::sample_standard_uniform(false).map(|v| v * &scale + &lower)
                } else { Ok(v.clone()) }
            }).collect()
        }),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())))
}

// utility trait to impute with a constant, regardless of the representation of null
pub trait ImputeNull: Domain {
    type Inner;
    fn impute_null<'a>(default: &'a Self::Carrier, constant: &'a Self::Inner) -> &'a Self::Inner;
    fn new() -> Self;
}
// how to impute, when null represented as Option<T>
impl<T: Clone> ImputeNull for AllDomain<Option<T>> {
    type Inner = T;
    fn impute_null<'a>(default: &'a Self::Carrier, constant: &'a Self::Inner) -> &'a Self::Inner {
        default.as_ref().unwrap_or(constant)
    }
    fn new() -> Self { AllDomain::new() }
}
// how to impute, when null represented as T with internal nullity
impl<T: Float> ImputeNull for NullableDomain<AllDomain<T>> {
    type Inner = Self::Carrier;
    fn impute_null<'a>(default: &'a Self::Carrier, constant: &'a Self::Inner) -> &'a Self::Inner {
        if default.is_nan() {constant} else {default}
    }
    fn new() -> Self { NullableDomain::new(AllDomain::new()) }
}

/// A [`Transformation`] that imputes elementwise with a constant value.
/// Maps a Vec<Option<T>> -> Vec<T> if input domain is AllDomain<Option<T>>,
///     or Vec<T> -> Vec<T> if input domain is NullableDomain<AllDomain<T>>
pub fn make_impute_constant<DAtom, M>(
    constant: DAtom::Inner
) -> Fallible<Transformation<VectorDomain<DAtom>, VectorDomain<AllDomain<DAtom::Inner>>, M, M>>
    where M: DatasetMetric<Distance=u32>,
          DAtom: ImputeNull,
          DAtom::Inner: 'static + Clone,
          M::Distance: One + DistanceConstant {

    Ok(Transformation::new(
        VectorDomain::new(DAtom::new()),
        VectorDomain::new_all(),
        Function::new(move |arg: &Vec<DAtom::Carrier>| arg.iter()
            .map(|v| DAtom::impute_null(v, &constant))
            .cloned().collect()),
        M::default(),
        M::default(),
        StabilityRelation::new_from_constant(M::Distance::one())))
}


#[cfg(test)]
mod test {
    use crate::dist::HammingDistance;
    use crate::error::ExplainUnwrap;
    use crate::trans::{make_impute_uniform_float, make_impute_constant};

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
        let imputer = make_impute_constant::<HammingDistance, _>("IMPUTED".to_string()).unwrap_test();

        let result = imputer.function.eval(&vec![Some("A".to_string()), None]).unwrap_test();

        assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
        assert!(imputer.stability_relation
            .eval(&1, &1).unwrap_test());
    }
}