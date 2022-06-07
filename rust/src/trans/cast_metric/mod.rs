use crate::{
    error::Fallible, 
    core::{Transformation, DatasetMetric, Domain, Function, Metric, StabilityRelation},
    dist::{ChangeOneDistance, InsertDeleteDistance, SymmetricDistance, HammingDistance},
    samplers::GeneratorOpenSSL
};
use rand::seq::SliceRandom;

#[cfg(feature="ffi")]
mod ffi;

pub trait Shuffle {
    fn shuffle(&mut self) -> Fallible<()>;
}
impl<TI> Shuffle for Vec<TI> {
    fn shuffle(&mut self) -> Fallible<()> {
        let mut rng = GeneratorOpenSSL::new();
        SliceRandom::shuffle(self.as_mut_slice(), &mut rng);
        rng.error
    }
}


pub trait MetricCast<MI: Metric, T>: Metric {
    fn function(arg: &T) -> Fallible<T>;
    fn stability_relation() -> StabilityRelation<MI, Self>;
}

macro_rules! impl_cast_dataset_metric {
    ($mi:ty, $mo:ty, false, $relation:expr) => (
         impl<T: Clone> MetricCast<$mi, T> for $mo {
            fn function(arg: &T) -> Fallible<T> {
                Ok(arg.clone())
            }
            fn stability_relation() -> StabilityRelation<$mi, Self> {
                $relation
            }
        }
    );
    ($mi:ty, $mo:ty, true, $relation:expr) => (
        impl<T: Clone + Shuffle> MetricCast<$mi, T> for $mo {
            fn function(arg: &T) -> Fallible<T> {
                let mut arg = arg.clone();
                arg.shuffle()?;
                Ok(arg)
            }
            fn stability_relation() -> StabilityRelation<$mi, Self> {
                $relation
            }
        }
    )
}

// // both are unordered, so no shuffle necessary. One change results in an add and delete
impl_cast_dataset_metric!(ChangeOneDistance, SymmetricDistance, false, StabilityRelation::new_from_constant(2));
// can't allow this, only well-defined on SizedDomains, not all D.
// impl_cast_dataset_metric!(SymmetricDistance, ChangeOneDistance, false, StabilityRelation::new_from_forward(|d_in: &IntDistance| d_in.inf_div(&2)));

impl_cast_dataset_metric!(InsertDeleteDistance, SymmetricDistance, false, StabilityRelation::new_from_constant(1));
impl_cast_dataset_metric!(SymmetricDistance, InsertDeleteDistance, true, StabilityRelation::new_from_constant(1));

impl_cast_dataset_metric!(HammingDistance, InsertDeleteDistance, false, StabilityRelation::new_from_constant(2));
// can't allow this, only well-defined on SizedDomains, not all D.
// impl_cast_dataset_metric!(InsertDeleteDistance, HammingDistance, false, StabilityRelation::new_from_forward(|d_in: &IntDistance| d_in.inf_div(&2)));

impl_cast_dataset_metric!(ChangeOneDistance, HammingDistance, false, StabilityRelation::new_from_constant(1));
impl_cast_dataset_metric!(HammingDistance, ChangeOneDistance, true, StabilityRelation::new_from_constant(1));

impl_cast_dataset_metric!(ChangeOneDistance, InsertDeleteDistance, true, StabilityRelation::new_from_constant(2));
// can't allow this, only well-defined on SizedDomains, not all D.
// impl_cast_dataset_metric!(InsertDeleteDistance, ChangeOneDistance, false, StabilityRelation::new_from_forward(|d_in: &IntDistance| d_in.inf_div(&2)));

impl_cast_dataset_metric!(HammingDistance, SymmetricDistance, false, StabilityRelation::new_from_constant(2));
// can't allow this, only well-defined on SizedDomains, not all D.
// impl_cast_dataset_metric!(SymmetricDistance, HammingDistance, true, StabilityRelation::new_from_forward(|d_in: &IntDistance| d_in.inf_div(&2)));


// no-op self casts
macro_rules! impl_cast_dataset_metric_self {
    ($($m:ty)+) => ($(impl_cast_dataset_metric!{$m, $m, false, StabilityRelation::new_from_constant(1)})+)
}
impl_cast_dataset_metric_self!(SymmetricDistance ChangeOneDistance HammingDistance InsertDeleteDistance);

pub fn make_cast_metric<D, MI, MO>(
    domain: D
) -> Fallible<Transformation<D, D, MI, MO>>
    where D: Domain + Clone,
          D::Carrier: Clone,
          MI: DatasetMetric, MO: DatasetMetric,
          MO: MetricCast<MI, D::Carrier> {

    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new_fallible(|val: &D::Carrier| MO::function(val)),
        MI::default(),
        MO::default(),
        MO::stability_relation()
    ))
}

#[cfg(test)]
mod test {
    use crate::dom::{VectorDomain, AllDomain};

    use super::*;
    
    #[test]
    fn test_cast_metric() -> Fallible<()> {
        let data = vec!["abc".to_string(), "1".to_string(), "1.".to_string()];
        let caster = make_cast_metric::<VectorDomain<AllDomain<_>>, ChangeOneDistance, SymmetricDistance>(VectorDomain::new_all())?;
        let _res = caster.invoke(&data)?;
        assert!(!caster.check(&1, &1)?);
        assert!(caster.check(&1, &2)?);
        Ok(())
    }
}