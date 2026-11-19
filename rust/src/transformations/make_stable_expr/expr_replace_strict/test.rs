use polars_plan::plans::{DynListLiteralValue, DynLiteralValue};

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::make_stable_lazyframe;

use super::*;
use crate::transformations::make_stable_expr::test_helper::replace::check_replace_expr;

fn check_replace_strict(
    old: Expr,
    new: Expr,
    default: Option<Expr>,
    output_dtype: Option<DataType>,
    expected: Option<(Series, bool)>,
) -> Fallible<()> {
    check_replace_strict_on_input(
        &[Some(1), Some(2), Some(3)],
        old,
        new,
        default,
        output_dtype,
        expected,
    )
}

fn check_replace_strict_on_input(
    input: &[Option<i32>],
    old: Expr,
    new: Expr,
    default: Option<Expr>,
    output_dtype: Option<DataType>,
    expected: Option<(Series, bool)>,
) -> Fallible<()> {
    check_replace_expr(
        input,
        col("").replace_strict(old, new, default, output_dtype),
        expected,
    )
}

#[test]
fn test_make_expr_replace_strict_standard_scalar() -> Fallible<()> {
    // A non-null default removes nullability; without a default, unmatched values become null.
    check_replace_strict(
        lit(1),
        lit(true),
        Some(lit(false)),
        Some(DataType::Boolean),
        Some((Series::new("".into(), [true, false, false]), false)),
    )?;

    check_replace_strict(
        lit(1),
        lit(true),
        Some(lit(false)),
        None,
        Some((Series::new("".into(), [true, false, false]), false)),
    )?;

    check_replace_strict(
        lit(1),
        lit(true),
        None,
        None,
        Some((Series::new("".into(), [Some(true), None, None]), true)),
    )
}

#[test]
fn test_make_expr_replace_strict_return_dtype() -> Fallible<()> {
    check_replace_strict(
        lit(1i32),
        lit(2i32),
        Some(lit(3i32)),
        Some(DataType::String),
        Some((Series::new("".into(), ["2", "3", "3"]), false)),
    )
}

#[test]
fn test_make_expr_replace_strict_return_dtype_with_string_default() -> Fallible<()> {
    check_replace_strict(
        lit(1i32),
        lit(2i32),
        Some(lit("missing")),
        Some(DataType::String),
        Some((Series::new("".into(), ["2", "missing", "missing"]), false)),
    )
}

#[test]
fn test_make_expr_replace_strict_materializes_inferred_integer_dtype() -> Fallible<()> {
    check_replace_strict(
        lit(1i32),
        lit(2i32),
        Some(lit(3i32)),
        None,
        Some((Series::new("".into(), [2i32, 3, 3]), false)),
    )?;
    check_replace_strict(
        lit(1i32),
        lit(-2i32),
        Some(lit(3i32)),
        None,
        Some((Series::new("".into(), [-2i32, 3, 3]), false)),
    )?;
    check_replace_strict(
        lit(1i32),
        lit(Series::new("".into(), [2i64])),
        Some(lit(-3i32)),
        None,
        Some((Series::new("".into(), [2i64, -3, -3]), false)),
    )
}

#[test]
fn test_make_expr_replace_strict_inferred_dtype_with_default() -> Fallible<()> {
    let categories = Categories::global();
    let new = Series::new("".into(), ["A"]).cast(&DataType::Categorical(
        categories.clone(),
        categories.mapping(),
    ))?;
    check_replace_strict(
        lit(1i32),
        lit(new),
        Some(lit("fallback")),
        None,
        Some((Series::new("".into(), ["A", "fallback", "fallback"]), false)),
    )
}

#[test]
fn test_make_expr_replace_strict_with_null_input() -> Fallible<()> {
    check_replace_strict_on_input(
        &[Some(1), None, Some(3)],
        lit(1i32),
        lit(4i32),
        Some(lit(0i32)),
        None,
        Some((Series::new("".into(), [4i32, 0, 0]), false)),
    )
}

#[test]
fn test_make_expr_replace_strict_standard_series() -> Fallible<()> {
    // Null replacements and defaults determine output nullability.
    check_replace_strict(
        lit(Series::new("".into(), vec![1, 2])),
        lit(Series::new("".into(), vec![Some("A"), None])),
        None,
        None,
        Some((Series::new("".into(), [Some("A"), None, None]), true)),
    )?;

    check_replace_strict(
        lit(Series::new("".into(), vec![1, 2])),
        lit(Series::new("".into(), vec![Some("A"), Some("B")])),
        Some(lit("C")),
        None,
        Some((
            Series::new("".into(), [Some("A"), Some("B"), Some("C")]),
            false,
        )),
    )
}

#[test]
fn test_make_expr_replace_strict_impute_series() -> Fallible<()> {
    // The unmatched input value remains null, even when an input null is replaced.
    check_replace_strict_on_input(
        &[Some(1), None, Some(3)],
        lit(Series::new("".into(), vec![Some(1), None])),
        lit(Series::new("".into(), vec![10, 0])),
        None,
        None,
        Some((Series::new("".into(), [Some(10), Some(0), None]), true)),
    )
}

#[test]
fn test_replace_strict_rejects_nested_list_literal() -> Fallible<()> {
    let nested = Expr::Literal(LiteralValue::Dyn(DynLiteralValue::List(
        DynListLiteralValue::List(
            vec![Some(DynListLiteralValue::Int(
                vec![Some(2_i128)].into_boxed_slice(),
            ))]
            .into_boxed_slice(),
        ),
    )));
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;

    for arg_pos in 1..=3 {
        let mut input = vec![col(""), lit(1i32), lit(2i32), lit(0i32)];
        input[arg_pos] = nested.clone();

        assert!(
            make_expr_replace_strict(
                domain.clone().row_by_row(),
                FrameDistance(SymmetricDistance),
                Expr::Function {
                    input,
                    function: FunctionExpr::ReplaceStrict { return_dtype: None },
                },
            )
            .is_err(),
            "nested list argument {arg_pos} must be rejected",
        );
    }
    Ok(())
}

#[test]
fn test_make_expr_replace_strict_wrong_dtype() -> Fallible<()> {
    check_replace_strict(lit("A"), lit(45), None, None, None)
}

#[test]
fn test_replace_strict_rejects_categorical() -> Fallible<()> {
    let categorical = Series::new("data".into(), ["A", "B"]).cast(&DataType::Categorical(
        Categories::global(),
        Categories::global().mapping(),
    ))?;
    let lf = DataFrame::new(vec![categorical.into_column()])?.lazy();
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        crate::domains::CategoricalDomain,
    )])?;
    assert!(
        make_stable_lazyframe(
            domain,
            FrameDistance(SymmetricDistance),
            lf.with_column(col("data").replace_strict(
                lit(Series::new("".into(), ["A", "B"])),
                lit(Series::new("".into(), ["X", "Y"])),
                None,
                None::<DataType>,
            )),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn test_replace_strict_rejects_nested_categorical() -> Fallible<()> {
    let categories = Categories::global();
    let categorical = DataType::Categorical(categories.clone(), categories.mapping());
    let nested = DataType::Array(Box::new(categorical), 2);
    let schema = Schema::from_iter([Field::new("data".into(), nested)]);
    let domain = LazyFrameDomain::new_from_schema(schema)?;
    let err = make_expr_replace_strict(
        domain.row_by_row(),
        FrameDistance(SymmetricDistance),
        col("data").replace_strict(lit(1i32), lit(2i32), Some(lit(3i32)), None::<DataType>),
    )
    .map(|_| ())
    .unwrap_err();
    assert!(
        err.message
            .as_deref()
            .unwrap_or_default()
            .contains("categorical"),
        "{err:?}"
    );
    Ok(())
}

#[test]
fn test_replace_strict_rejects_categorical_output() -> Fallible<()> {
    let categories = Categories::global();
    let dtype = DataType::Categorical(categories.clone(), categories.mapping());
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "",
        OptionDomain::new(AtomDomain::<i32>::default()),
    )])?;
    assert!(
        make_expr_replace_strict(
            domain.row_by_row(),
            FrameDistance(SymmetricDistance),
            col("").replace_strict(lit(1i32), lit("X"), Some(lit("fallback")), Some(dtype)),
        )
        .is_err()
    );
    Ok(())
}
