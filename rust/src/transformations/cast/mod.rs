#[cfg(feature = "ffi")]
mod ffi;

use opendp_derive::bootstrap;

use crate::core::Transformation;
use crate::domains::{AtomDomain, OptionDomain, VectorDomain};
use crate::error::Fallible;
use crate::metrics::SymmetricDistance;
use crate::traits::{CheckAtom, InherentNull, RoundCast};
use crate::transformations::make_row_by_row;

#[bootstrap(features("contrib"))]
/// Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
/// For each element, failure to parse results in `None`, else `Some(out)`.
///
/// Can be chained with `make_impute_constant` or `make_drop_null` to handle nullity.
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast<TIA, TOA>() -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<OptionDomain<AtomDomain<TOA>>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + CheckAtom,
{
    make_row_by_row(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        OptionDomain::new(AtomDomain::default()),
        |v| {
            TOA::round_cast(v.clone())
                .ok()
                .and_then(|v| if v.is_null() { None } else { Some(v) })
        },
    )
}

#[bootstrap(features("contrib"))]
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
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_default<TIA, TOA>() -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<TOA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + Default + CheckAtom,
{
    make_row_by_row(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        AtomDomain::default(),
        |v| TOA::round_cast(v.clone()).unwrap_or_default(),
    )
}

#[bootstrap(features("contrib"))]
/// Make a Transformation that casts a vector of data from type `TIA` to a type that can represent nullity `TOA`.
/// If cast fails, fill with `TOA`'s null value.
///
/// | `TIA`  | `TIA::default()` |
/// | ------ | ---------------- |
/// | float  | NaN              |
///
/// # Generics
/// * `TIA` - Atomic Input Type to cast from
/// * `TOA` - Atomic Output Type to cast into
pub fn make_cast_inherent<TIA, TOA>() -> Fallible<
    Transformation<
        VectorDomain<AtomDomain<TIA>>,
        VectorDomain<AtomDomain<TOA>>,
        SymmetricDistance,
        SymmetricDistance,
    >,
>
where
    TIA: 'static + Clone + CheckAtom,
    TOA: 'static + RoundCast<TIA> + InherentNull + CheckAtom,
{
    make_row_by_row(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
        AtomDomain::new_nullable(),
        |v| TOA::round_cast(v.clone()).unwrap_or(TOA::NULL),
    )
}

#[cfg(test)]
mod tests {
    use crate::error::ExplainUnwrap;

    use super::*;

    #[test]
    fn test_cast() -> Fallible<()> {
        let data = vec![1., 1e10, 0.5, f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast::<f64, i64>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![Some(1), Some(10000000000), Some(0), None, None, None]
        );

        let caster = make_cast::<f64, u8>()?;
        assert_eq!(
            caster.invoke(&vec![-1., f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![None; 4]
        );
        Ok(())
    }

    #[test]
    fn test_cast_combinations() {
        macro_rules! test_pair {
            ($from:ty, $to:ty) => {
                let caster = make_cast::<$from, $to>().unwrap_test();
                caster.invoke(&vec![<$from>::default()]).unwrap_test();
                let caster = make_cast::<$from, $to>().unwrap_test();
                caster.invoke(&vec![<$from>::default()]).unwrap_test();
            };
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
        test_cartesian! {[];[u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String, bool]}
    }

    #[test]
    fn test_cast_default_unsigned() -> Fallible<()> {
        let caster = make_cast_default::<f64, u8>()?;
        assert_eq!(caster.invoke(&vec![-1.])?, vec![u8::default()]);
        Ok(())
    }

    #[test]
    fn test_cast_default_parse() -> Fallible<()> {
        let data = vec![
            "2".to_string(),
            "3".to_string(),
            "a".to_string(),
            "".to_string(),
        ];

        let caster = make_cast_default::<String, u8>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![2, 3, u8::default(), u8::default()]
        );

        let caster = make_cast_default::<String, f64>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![2., 3., f64::default(), f64::default()]
        );
        Ok(())
    }

    #[test]
    fn test_cast_default_floats() -> Fallible<()> {
        let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast_default::<f64, String>()?;
        assert_eq!(
            caster.invoke(&data)?,
            vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]
        );

        let caster = make_cast_default::<f64, u8>()?;
        assert_eq!(
            caster.invoke(&vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![u8::default(), u8::default(), u8::default()]
        );

        let data = vec![
            "1e+2",
            "1e2",
            "1e+02",
            "1.e+02",
            "1.0E+02",
            "1.0E+00002",
            "01.E+02",
            "1.0E2",
        ]
        .into_iter()
        .map(|v| v.to_string())
        .collect();
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
}
