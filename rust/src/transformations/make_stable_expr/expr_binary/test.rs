use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};
use core::f64;
use std::ops::{Add, Div, Mul, Rem, Sub};

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

fn get_i32_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<i32>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<i32>::default())),
    ])?;

    let lf = df!(
        "L" => [Some(1), Some(1), Some(1)],
        "R" => [Some(0), Some(1), None],
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
            Some(true), None, Some(false), None, None
        ],
        "R" => [
            Some(true), Some(false), Some(true), Some(false),
            None, Some(true), None, Some(false), None
        ],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

fn get_string_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("L", AtomDomain::<String>::default()),
        SeriesDomain::new("R", OptionDomain::new(AtomDomain::<String>::default())),
    ])?;

    let lf = df!(
        "L" => ["".to_string(), "A".to_string()],
        "R" => [Some("1".to_string()), None],
    )?
    .lazy();

    Ok((lf_domain, lf))
}

macro_rules! test_binary {
    // (the function to call to get the data, the operation to test, the expected data output)
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
            lf.clone()
                .with_columns([col("L").$op(col("R")).alias("O")])
                .collect()?
        );
        let t_op = col("L")
            .$op(col("R"))
            .make_stable(expr_domain.clone(), FrameDistance(SymmetricDistance))?;
        let output_series = &t_op.output_domain.column;
        assert_eq!(&*output_series.name, "L");

        let out = lf
            .select([col("L").$op(col("R")).alias("out").null_count()])
            .collect()?
            .column("out")?
            .u32()?
            .get(0)
            .unwrap();

        assert_eq!(output_series.nullable, (out > 0));
        Fallible::Ok(())
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
            // NaN is equal to NaN: https://docs.pola.rs/user-guide/concepts/data-types-and-structures/#floating-point-numbers
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
fn test_eq_missing() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        eq_missing,
        [
            true,
            false, // NaN is equal to NaN: https://docs.pola.rs/user-guide/concepts/data-types-and-structures/#floating-point-numbers
            true, false, true, false, true, false,
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
fn test_neq_validity() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        neq_missing,
        [false, true, false, true, false, true, false, true,]
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
            // nan is equal to NaN, zero is lte NaN: https://docs.pola.rs/user-guide/concepts/data-types-and-structures/#floating-point-numbers
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
            // nan is equal to NaN: https://docs.pola.rs/user-guide/concepts/data-types-and-structures/#floating-point-numbers
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
            None,
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
            None,
            None,
        ]
    )
}

#[test]
fn test_add() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        add,
        [
            Some(0.),
            Some(1.),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::INFINITY),
            Some(f64::INFINITY)
        ]
    )?;
    test_binary!(get_i32_data, add, [Some(1), Some(2), None])?;
    test_binary!(get_string_data, add, [Some("1".to_string()), None])?;
    Ok(())
}

#[test]
fn test_sub() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        sub,
        [
            Some(0.),
            Some(1.),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::NAN),
            Some(-f64::INFINITY)
        ]
    )?;
    test_binary!(get_i32_data, sub, [Some(1), Some(0), None])?;
    Ok(())
}

#[test]
fn test_mul() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        mul,
        [
            Some(0.),
            Some(0.),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::INFINITY),
            Some(f64::NAN)
        ]
    )?;
    test_binary!(get_i32_data, mul, [Some(0), Some(1), None])?;
    Ok(())
}

#[test]
fn test_div() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        div,
        [
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::NAN),
            Some(0.)
        ]
    )?;
    test_binary!(get_i32_data, div, [None, Some(1), None])?;
    Ok(())
}

#[test]
fn test_floor_div() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        floor_div,
        [
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::NAN),
            Some(0.)
        ]
    )?;
    test_binary!(get_i32_data, floor_div, [None, Some(1), None])?;
    Ok(())
}

#[test]
fn test_rem() -> Fallible<()> {
    test_binary!(
        get_f64_data,
        rem,
        [
            Some(f64::NAN),
            Some(f64::NAN),
            Some(f64::NAN),
            Some(f64::NAN),
            None,
            None,
            Some(f64::NAN),
            Some(f64::NAN)
        ]
    )?;
    test_binary!(get_i32_data, rem, [None, Some(0), None])?;
    Ok(())
}

#[test]
fn test_overflow() -> Fallible<()> {
    // ensures behavior of arithmetic is as expected when overflow occurs
    let data = df!(
        "x" => [i32::MAX, i32::MIN],
        "add" => [1, -1],
        "sub" => [-1, 1],
        "mul" => [2, 2],
    )?;
    let expected = df!(
        "add" => [i32::MIN, i32::MAX],
        "sub" => [i32::MIN, i32::MAX],
        "mul" => [-2, 0],
    )?;
    let observed = data
        .lazy()
        .select([
            (col("x") + col("add")).alias("add"),
            (col("x") - col("sub")).alias("sub"),
            (col("x") * col("mul")).alias("mul"),
        ])
        .collect()?;
    assert_eq!(observed, expected);
    Ok(())
}
