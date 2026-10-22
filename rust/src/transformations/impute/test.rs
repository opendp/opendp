use crate::metrics::SymmetricDistance;

use super::*;

fn with_imputation_timeout(test: impl FnOnce() -> Fallible<()> + Send + 'static) {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = test().map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
    // Guard against nontermination if narrow-bound sampling regresses.
    receiver
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("uniform imputation did not complete")
        .expect("uniform imputation failed");
}

#[test]
fn test_impute_uniform() -> Fallible<()> {
    let imputer = make_impute_uniform_float(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        (2.0, 3.0),
    )?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![1.0, f64::NAN])?;

    assert_eq!(result[0], 1.);
    assert!((2.0..=3.0).contains(&result[1]));
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

macro_rules! test_impute_uniform_float {
    ($float:ty, $range_test:ident, $preserve_test:ident) => {
        #[test]
        fn $range_test() -> Fallible<()> {
            with_imputation_timeout(|| {
                let one: $float = 1.0;
                let below_one = <$float>::from_bits(one.to_bits() - 1);
                let above_one = <$float>::from_bits(one.to_bits() + 1);
                let subnormal = <$float>::from_bits(1);
                let high = <$float>::MAX / 2.0;
                let above_high = <$float>::from_bits(high.to_bits() + 1);
                let cases = [
                    ("adjacent above one", one, above_one),
                    ("adjacent below one", below_one, one),
                    ("adjacent below negative one", -above_one, -one),
                    ("adjacent above negative one", -one, -below_one),
                    ("positive subnormal", 0.0, subnormal),
                    ("negative subnormal", -subnormal, 0.0),
                    ("subnormal crossing zero", -subnormal, subnormal),
                    ("high positive adjacent", high, above_high),
                    ("high negative adjacent", -above_high, -high),
                    ("wide finite interval", -high / 2.0, high / 2.0),
                    ("maximum positive width", 0.0, <$float>::MAX),
                    ("maximum negative width", -<$float>::MAX, 0.0),
                    ("maximum width crossing zero", -high, high),
                    ("ordinary positive", 2.0, 3.0),
                    ("ordinary negative", -3.0, -2.0),
                    ("ordinary crossing zero", -0.5, 0.5),
                ];

                for (case, lower, upper) in cases {
                    // Overflowing widths and non-finite bounds are a separate case.
                    assert!((upper - lower).is_finite(), "{case}");
                    let imputer = make_impute_uniform_float(
                        VectorDomain::new(AtomDomain::<$float>::default()),
                        SymmetricDistance,
                        (lower, upper),
                    )?;
                    let input = vec![<$float>::NAN; 16];
                    assert!(imputer.input_domain.member(&input)?, "{case}");

                    let result = imputer.invoke(&input)?;

                    assert_eq!(result.len(), input.len(), "{case}");
                    for value in result.iter() {
                        assert!(value.is_finite(), "{case}: {value}");
                        assert!((lower..=upper).contains(value), "{case}: {value}");
                    }
                    assert!(!imputer.output_domain.element_domain.nan(), "{case}");
                    assert!(imputer.output_domain.member(&result)?, "{case}");
                    assert!(imputer.check(&0, &0)?, "{case}");
                    assert!(imputer.check(&1, &1)?, "{case}");
                    assert!(imputer.check(&7, &7)?, "{case}");
                }
                Ok(())
            });
            Ok(())
        }

        #[test]
        fn $preserve_test() -> Fallible<()> {
            let one: $float = 1.0;
            let imputer = make_impute_uniform_float(
                VectorDomain::new(AtomDomain::<$float>::default()),
                SymmetricDistance,
                (one, <$float>::from_bits(one.to_bits() + 1)),
            )?;
            let input = vec![
                <$float>::NEG_INFINITY,
                -2.0,
                -0.0,
                0.0,
                one,
                <$float>::MAX,
                <$float>::INFINITY,
            ];
            assert!(imputer.input_domain.member(&input)?);
            let result = imputer.invoke(&input)?;
            assert_eq!(
                result
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                input
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
            );
            assert!(imputer.output_domain.member(&result)?);

            let empty = vec![];
            assert!(imputer.invoke(&empty)?.is_empty());
            assert!(imputer.output_domain.member(&empty)?);
            assert!(imputer.check(&0, &0)?);
            assert!(imputer.check(&1, &1)?);
            Ok(())
        }
    };
}

test_impute_uniform_float!(
    f32,
    test_impute_uniform_f32_finite_intervals,
    test_impute_uniform_f32_preserves_non_nan
);
test_impute_uniform_float!(
    f64,
    test_impute_uniform_f64_finite_intervals,
    test_impute_uniform_f64_preserves_non_nan
);

#[test]
fn test_impute_constant_option() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(OptionDomain::new(AtomDomain::default())),
        SymmetricDistance,
        "IMPUTED".to_string(),
    )?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![Some("A".to_string()), None])?;

    assert_eq!(result, vec!["A".to_string(), "IMPUTED".to_string()]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_constant_inherent() -> Fallible<()> {
    let imputer = make_impute_constant(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance,
        12.,
    )?;

    let result = imputer.invoke(&vec![f64::NAN, 23.])?;

    assert_eq!(result, vec![12., 23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}

#[test]
fn test_impute_drop_option() -> Fallible<()> {
    let imputer = make_drop_null(
        VectorDomain::new(OptionDomain::default()),
        SymmetricDistance,
    )?;

    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![Some(f64::NAN), Some(23.), None])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
#[test]
fn test_impute_drop_inherent() -> Fallible<()> {
    let imputer = make_drop_null(VectorDomain::new(AtomDomain::default()), SymmetricDistance)?;
    assert!(!imputer.output_domain.element_domain.nan());

    let result = imputer.invoke(&vec![f64::NAN, 23.])?;

    assert_eq!(result, vec![23.]);
    assert!(imputer.check(&1, &1)?);
    Ok(())
}
