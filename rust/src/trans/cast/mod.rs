#[cfg(feature="ffi")]
mod ffi;

use crate::core::{DatasetMetric, Domain, Function, StabilityRelation, Transformation};
use crate::dist::{SubstituteDistance, SymmetricDistance, IntDistance};
use crate::dom::{AllDomain, InherentNull, InherentNullDomain, OptionNullDomain, VectorDomain};
use crate::error::Fallible;
use crate::traits::{RoundCast, CheckNull};
use crate::trans::make_row_by_row;

/// A [`Transformation`] that casts elements between types
/// Maps a Vec<TIA> -> Vec<Option<TOA>>
pub fn make_cast<TIA, TOA>() -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<OptionNullDomain<AllDomain<TOA>>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + Clone + CheckNull, TOA: 'static + RoundCast<TIA> + CheckNull {
    make_row_by_row(
        AllDomain::new(),
        OptionNullDomain::new(AllDomain::new()),
        |v| TOA::round_cast(v.clone()).ok()
            .and_then(|v| if v.is_null() {None} else {Some(v)}))
}

/// A [`Transformation`] that casts elements between types. Fills with TO::default if parsing fails.
/// Maps a Vec<TIA> -> Vec<TOA>
pub fn make_cast_default<TIA, TOA>() -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<AllDomain<TOA>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + Clone + CheckNull, TOA: 'static + RoundCast<TIA> + Default + CheckNull {
    make_row_by_row(
        AllDomain::new(),
        AllDomain::new(),
        |v| TOA::round_cast(v.clone()).unwrap_or_default())
}

/// A [`Transformation`] that casts elements to a type that has an inherent representation of nullity.
/// Maps a Vec<TI> -> Vec<TO>
pub fn make_cast_inherent<TIA, TOA>(
) -> Fallible<Transformation<VectorDomain<AllDomain<TIA>>, VectorDomain<InherentNullDomain<AllDomain<TOA>>>, SymmetricDistance, SymmetricDistance>>
    where TIA: 'static + Clone + CheckNull, TOA: 'static + RoundCast<TIA> + InherentNull + CheckNull {
    make_row_by_row(
        AllDomain::new(),
        InherentNullDomain::new(AllDomain::new()),
        |v| TOA::round_cast(v.clone()).unwrap_or(TOA::NULL))
}

pub trait DatasetMetricCast {
    fn stability_constant() -> IntDistance;
}

macro_rules! impl_metric_cast {
    ($ty:ty, $constant:literal) => {
         impl DatasetMetricCast for $ty {
            fn stability_constant() -> IntDistance {
                $constant
            }
        }
    }
}
impl_metric_cast!((SubstituteDistance, SymmetricDistance), 2);
impl_metric_cast!((SymmetricDistance, SubstituteDistance), 1);
impl_metric_cast!((SymmetricDistance, SymmetricDistance), 1);
impl_metric_cast!((SubstituteDistance, SubstituteDistance), 1);

pub fn make_cast_metric<D, MI, MO>(
    domain: D
) -> Fallible<Transformation<D, D, MI, MO>>
    where D: Domain + Clone,
          D::Carrier: Clone,
          MI: DatasetMetric, MO: DatasetMetric,
          (MI, MO): DatasetMetricCast {

    Ok(Transformation::new(
        domain.clone(),
        domain,
        Function::new(|val: &D::Carrier| val.clone()),
        MI::default(),
        MO::default(),
        StabilityRelation::new_from_constant(<(MI, MO)>::stability_constant())
    ))
}

#[cfg(test)]
mod tests {
    use crate::dist::{SubstituteDistance, SymmetricDistance};
    use crate::error::ExplainUnwrap;

    use super::*;


    #[test]
    fn test_cast() -> Fallible<()> {
        let data = vec![1., 1e10, 0.5, f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast::<f64, i64>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![Some(1), Some(10000000000), Some(0), None, None, None]);

        let caster = make_cast::<f64, u8>()?;
        assert_eq!(
            caster.invoke(&vec![-1., f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![None; 4]);
        Ok(())
    }

    #[test]
    fn test_cast_combinations() {
        macro_rules! test_pair {
            ($from:ty, $to:ty) => {
                let caster = make_cast::<$from, $to>().unwrap_test();
                caster.invoke(&vec!(<$from>::default())).unwrap_test();
                let caster = make_cast::<$from, $to>().unwrap_test();
                caster.invoke(&vec!(<$from>::default())).unwrap_test();
            }
        }
        macro_rules! test_cartesian {
            ([];[$first:ty, $($end:ty),*]) => {
                test_pair!($first, $first);
                $(test_pair!($first, $end);)*

                test_cartesian!{[$first];[$($end),*]}
            };
            ([$($start:ty),*];[$mid:ty, $($end:ty),*]) => {
                $(test_pair!($mid, $start);)*
                test_pair!($mid, $mid);
                $(test_pair!($mid, $end);)*

                test_cartesian!{[$($start),*, $mid];[$($end),*]}
            };
            ([$($start:ty),*];[$last:ty]) => {
                test_pair!($last, $last);
                $(test_pair!($last, $start);)*
            };
        }
        test_cartesian!{[];[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, bool]}
    }

    #[test]
    fn test_cast_default_unsigned() -> Fallible<()> {
        let caster = make_cast_default::<f64, u8>()?;
        assert_eq!(caster.invoke(&vec![-1.])?, vec![u8::default()]);
        Ok(())
    }

    #[test]
    fn test_cast_default_parse() -> Fallible<()> {
        let data = vec!["2".to_string(), "3".to_string(), "a".to_string(), "".to_string()];

        let caster = make_cast_default::<String, u8>()?;
        assert_eq!(caster.invoke(&data)?, vec![2, 3, u8::default(), u8::default()]);

        let caster = make_cast_default::<String, f64>()?;
        assert_eq!(caster.invoke(&data)?, vec![2., 3., f64::default(), f64::default()]);
        Ok(())
    }

    #[test]
    fn test_cast_default_floats() -> Fallible<()> {
        let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast_default::<f64, String>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]);

        let caster = make_cast_default::<f64, u8>()?;
        assert_eq!(
            caster.invoke(&vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![u8::default(), u8::default(), u8::default()]);

        let data = vec!["1e+2", "1e2", "1e+02", "1.e+02", "1.0E+02", "1.0E+00002", "01.E+02", "1.0E2"]
            .into_iter().map(|v| v.to_string()).collect();
        let caster = make_cast_default::<String, f64>()?;
        assert!(caster.invoke(&data)?.into_iter().all(|v| v == 100.));
        Ok(())
    }

    #[test]
    fn test_cast_inherent() -> Fallible<()> {
        let data = vec!["abc".to_string(), "1".to_string(), "1.".to_string()];
        let caster = make_cast_inherent::<String, f64>()?;
        let res = caster.invoke(&data)?;
        assert!(res[0].is_nan());
        assert_eq!(res[1..], vec![1., 1.]);
        Ok(())
    }

    #[test]
    fn test_cast_metric() -> Fallible<()> {
        let data = vec!["abc".to_string(), "1".to_string(), "1.".to_string()];
        let caster = make_cast_metric::<VectorDomain<AllDomain<_>>, SubstituteDistance, SymmetricDistance>(VectorDomain::new_all())?;
        let _res = caster.invoke(&data)?;
        assert!(!caster.check(&1, &1)?);
        assert!(caster.check(&1, &2)?);
        Ok(())
    }
}
