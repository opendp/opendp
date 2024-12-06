use crate::domains::{LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;

use super::*;

fn get_f64_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<f64>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<f64>::default())),
    ])?;

    let lf = df!(
        "L" => [
            Some(0.), Some(1.),
            Some(f64::NAN), Some(0.),
            None, Some(0.),
            Some(f64::INFINITY), Some(0.)
        ],
        "R" => [
            Some(0.), Some(0.),
            Some(f64::NAN), Some(f64::NAN),
            None, None,
            Some(f64::INFINITY), Some(f64::INFINITY)
        ],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

fn get_bool_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<bool>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<bool>::default())),
    ])?;

    let lf = df!(
        "L" => [
            Some(true), Some(true), Some(false), Some(false),
            Some(true), None, Some(false), None
        ],
        "R" => [
            Some(true), Some(false), Some(true), Some(false),
            None, Some(true), None, Some(false)
        ],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

macro_rules! test_binary {
    ($get:ident, $op:ident, $expected:expr) => {{
        let (lf_domain, lf) = $get()?;
        let expr_domain = lf_domain.row_by_row();
        assert_eq!(
            lf.clone()
                .with_column(col("L").$op(col("R")))
                .collect()?
                .column("L")?,
            &Column::new("L".into(), $expected),
            "input: {:?}",
            lf.clone().collect()?
        );
        let t_op = col("L")
            .$op(col("R"))
            .make_stable(expr_domain.clone(), SymmetricDistance)?;
        let output_series = &t_op.output_domain.column;
        assert_eq!(&*output_series.name, "L");

        let out = lf
            .select([col("L").$op(col("R")).alias("out").null_count()])
            .collect()?
            .column("out")?
            .u32()?
            .get(0)
            .unwrap();

        assert!(output_series.nullable == (out > 0));
        Ok(())
    }};
}

#[test]
fn test_eq() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        eq,
        [
            Some(true),
            Some(false),
            // NaN is equal to NaN?
            Some(true),
            Some(false),
            None,
            None,
            Some(true),
            Some(false),
        ]
    )
}

#[test]
fn test_neq() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        neq,
        [
            Some(false),
            Some(true),
            Some(false),
            Some(true),
            None,
            None,
            Some(false),
            Some(true),
        ]
    )
}

#[test]
fn test_lt() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        lt,
        [
            Some(false),
            Some(false),
            // zero is less than NaN?!
            Some(false),
            Some(true),
            None,
            None,
            Some(false),
            Some(true),
        ]
    )
}

#[test]
fn test_lt_eq() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        lt_eq,
        [
            Some(true),
            Some(false),
            // nan is equal to NaN? zero is lte NaN?
            Some(true),
            Some(true),
            None,
            None,
            Some(true),
            Some(true),
        ]
    )
}

#[test]
fn test_gt() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        gt,
        [
            Some(false),
            Some(true),
            Some(false),
            Some(false),
            None,
            None,
            Some(false),
            Some(false),
        ]
    )
}

#[test]
fn test_gt_eq() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        gt_eq,
        [
            Some(true),
            Some(true),
            // nan is equal to NaN?
            Some(true),
            Some(false),
            None,
            None,
            Some(true),
            Some(false),
        ]
    )
}

#[test]
fn test_and() -> Fallible<()> {
    test_binary!(
        get_bool_data,
        and,
        [
            Some(true),
            Some(false),
            Some(false),
            Some(false),
            None,
            None,
            Some(false),
            Some(false),
        ]
    )
}

#[test]
fn test_or() -> Fallible<()> {
    test_binary!(
        get_bool_data,
        or,
        [
            Some(true),
            Some(true),
            Some(true),
            Some(false),
            Some(true),
            Some(true),
            None,
            None,
        ]
    )
}

#[test]
fn test_logical_and() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        logical_and,
        [
            Some(false),
            Some(false),
            // nan is truth-y
            Some(true),
            Some(false),
            None,
            Some(false),
            Some(true),
            Some(false)
        ]
    )
}

#[test]
fn test_logical_or() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        logical_or,
        [
            Some(false),
            Some(true),
            // nan is truth-y
            Some(true),
            Some(true),
            None,
            None,
            Some(true),
            Some(true)
        ]
    )
}

#[test]
fn test_xor() -> Fallible<()> {
    test_binary!(
        get_bool_data,
        xor,
        [
            Some(false),
            Some(true),
            Some(true),
            Some(false),
            // why not same behavior as or?
            None,
            None,
            None,
            None
        ]
    )
}
