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
mod tests {
    use crate::metrics::{HammingDistance, InsertDeleteDistance, SymmetricDistance};

    use super::*;

    #[test]
    fn test_cast() -> Fallible<()> {
        let data = vec![1., 1e10, 0.5, f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster = make_cast::<_, f64, i64>(
            VectorDomain::new(AtomDomain::default()),
            InsertDeleteDistance::default(),
        )?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![Some(1), Some(10000000000), Some(0), None, None, None]
        );

        let caster = make_cast::<_, f64, u8>(
            VectorDomain::new(AtomDomain::default()).with_size(4),
            HammingDistance::default(),
        )?;
        assert_eq!(
            caster.invoke(&vec![-1., f64::NAN, f64::NEG_INFINITY, f64::INFINITY])?,
            vec![None; 4]
        );
        Ok(())
    }

    #[test]
    fn test_cast_combinations() -> Fallible<()> {
        macro_rules! test_pair {
            ($from:ty, $to:ty) => {
                let caster = make_cast::<_, $from, $to>(
                    VectorDomain::new(AtomDomain::default()),
                    SymmetricDistance::default(),
                )?;
                caster.invoke(&vec![<$from>::default()])?;
                let caster = make_cast::<_, $from, $to>(
                    VectorDomain::new(AtomDomain::default()),
                    SymmetricDistance::default(),
                )?;
                caster.invoke(&vec![<$from>::default()])?;
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
        Ok(())
    }

    #[test]
    fn test_cast_default_unsigned() -> Fallible<()> {
        let caster =
            make_cast_default::<_, f64, u8>(Default::default(), SymmetricDistance::default())?;
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

        let caster =
            make_cast_default::<_, String, u8>(Default::default(), SymmetricDistance::default())?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![2, 3, u8::default(), u8::default()]
        );

        let caster =
            make_cast_default::<_, String, f64>(Default::default(), SymmetricDistance::default())?;
        assert_eq!(
            caster.invoke(&data)?,
            vec![2., 3., f64::default(), f64::default()]
        );
        Ok(())
    }

    #[test]
    fn test_cast_default_floats() -> Fallible<()> {
        let data = vec![f64::NAN, f64::NEG_INFINITY, f64::INFINITY];
        let caster =
            make_cast_default::<_, f64, String>(Default::default(), SymmetricDistance::default())?;
        assert_eq!(
            caster.invoke(&data)?,
            vec!["NaN".to_string(), "-inf".to_string(), "inf".to_string()]
        );

        let caster =
            make_cast_default::<_, f64, u8>(Default::default(), SymmetricDistance::default())?;
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
        let caster =
            make_cast_default::<_, String, f64>(Default::default(), SymmetricDistance::default())?;
        assert!(caster.invoke(&data)?.into_iter().all(|v| v == 100.));
        Ok(())
    }

    #[test]
    fn test_cast_inherent() -> Fallible<()> {
        let data = vec!["abc".to_string(), "1".to_string(), "1.".to_string()];
        let caster = make_cast_inherent::<_, String, f64>(
            VectorDomain::default(),
            SymmetricDistance::default(),
        )?;
        let res = caster.invoke(&data)?;
        assert!(res[0].is_nan());
        assert_eq!(res[1..], vec![1., 1.]);
        Ok(())
    }
}
