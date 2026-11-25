use super::*;

#[test]
fn test_permutation_checker_cast_and_fill_hardening() -> Fallible<()> {
    let schema = Schema::from_iter([Field::new("i".into(), DataType::Int32)]);
    for expr in [
        col("i")
            .cast_with_options(DataType::Int64, CastOptions::Overflowing)
            .sum(),
        col("i").strict_cast(DataType::Int64).sum(),
        col("i").fill_null(lit("not an integer")).sum(),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "unexpectedly accepted: {expr:?}"
        );
    }
    check_permutation_invariance(&col("i").cast(DataType::Int64).sum(), &schema)?;
    check_permutation_invariance(&col("i").fill_null(lit(0)).sum(), &schema)?;
    Ok(())
}

#[test]
fn test_permutation_checker_resize() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("A".into(), DataType::Int32),
        Field::new("B".into(), DataType::Int32),
    ]);
    assert_eq!(
        permutation_behavior(&col("A"), Resize::Ban, Some(&schema))?,
        PermutationBehavior::Equivariant
    );
    assert!(is_permutation_equivariant(&col("A")));
    assert!(is_permutation_equivariant(&len()));
    assert!(!is_permutation_equivariant(&col("A").sum()));
    assert!(is_permutation_equivariant(&as_struct(vec![
        col("A"),
        col("B")
    ])));
    assert!(!is_permutation_equivariant(
        &col("A").strict_cast(DataType::Int64)
    ));
    // Integer sum is a broadcastable invariant scalar.
    assert_eq!(
        permutation_behavior(&col("A").sum(), Resize::Ban, Some(&schema))?,
        PermutationBehavior::Invariant
    );
    // Data-dependent resizes and order-dependent intermediate results are rejected.
    assert!(permutation_behavior(&col("A").unique(), Resize::Ban, Some(&schema)).is_err());
    Ok(())
}

#[test]
fn test_permutation_checker_filter_input_alignment() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("A".into(), DataType::Int32),
        Field::new("B".into(), DataType::Boolean),
    ]);
    let predicate = col("B");
    check_permutation_invariance(&col("A").filter(predicate.clone()).sum(), &schema)?;

    // A data-dependent resize must not be combined with a predicate
    // evaluated over the original, unfiltered group.
    for expr in [
        col("A").drop_nulls().filter(predicate.clone()).sum(),
        col("A").filter(predicate.clone()).filter(predicate).sum(),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "unexpectedly accepted: {expr:?}"
        );
    }
    Ok(())
}

#[test]
fn test_permutation_checker_supported_aggregations() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("i".into(), DataType::Int32),
        Field::new("flag".into(), DataType::Boolean),
        Field::new("x".into(), DataType::Float64),
    ]);
    for expr in [
        col("i").sum(),
        col("i").count(),
        col("i").n_unique(),
        col("i").min(),
        col("i").max(),
        col("i").clip(lit(0), lit(100)).sum(),
        when(col("flag")).then(col("i")).otherwise(lit(0)).sum(),
        col("i").drop_nulls().min(),
        col("i").is_null().sum(),
        col("i").is_not_null().sum(),
        col("flag").any(false),
        col("flag").all(false),
    ] {
        check_permutation_invariance(&expr, &schema)?;
    }
    Ok(())
}

#[test]
fn test_permutation_checker_float_preprocessing() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("x".into(), DataType::Float64),
        Field::new("i".into(), DataType::Int64),
    ]);
    let sqrt = Expr::Function {
        input: vec![col("x")],
        function: FunctionExpr::Pow(PowFunction::Sqrt),
    };
    for expr in [
        col("x").abs(),
        -col("x"),
        col("x").floor(),
        col("x").ceil(),
        col("x").round(2, Default::default()),
        col("x").clip(lit(0.0), lit(100.0)),
        col("x").is_nan(),
        col("x").is_finite(),
        sqrt,
    ] {
        assert_eq!(
            permutation_behavior(&expr, Resize::Allow, Some(&schema))?,
            PermutationBehavior::Equivariant,
            "{expr:?}"
        );
    }
    for expr in [
        col("x").sum(),
        col("x").min(),
        col("x").max(),
        col("x").mean(),
        col("x").first(),
    ] {
        assert!(check_permutation_invariance(&expr, &schema).is_err());
    }
    // Integer division and integer unary overflow need independent certification.
    assert!(permutation_behavior(&(col("i") / col("i")), Resize::Allow, Some(&schema)).is_err());
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_unsafe_parameters() -> Fallible<()> {
    let categories = FrozenCategories::new(["low", "high"].into_iter())?;
    let schema = Schema::from_iter([
        Field::new(
            "level".into(),
            DataType::Enum(categories.clone(), categories.mapping().clone()),
        ),
        Field::new("text".into(), DataType::String),
        Field::new("i".into(), DataType::Int32),
        Field::new("lower".into(), DataType::Int32),
        Field::new("flag".into(), DataType::Boolean),
        Field::new(
            "category".into(),
            DataType::Categorical(Categories::global(), Categories::global().mapping()),
        ),
    ]);
    for expr in [
        col("i").cast(DataType::String).n_unique(),
        col("category").cast(DataType::UInt32).sum(),
        col("category").to_physical().sum(),
        col("level").lt(col("text")).sum(),
        col("i").clip(col("lower"), lit(10)).sum(),
        col("i").fill_null(col("lower")).sum(),
        col("i").filter(col("i")).sum(),
        col("i").sort(Default::default()).first(),
        col("i").shift(lit(1)).sum(),
        col("i").abs().sum(),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "unexpectedly accepted: {expr:?}"
        );
    }
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_nonlocal_or_fallible_functions() -> Fallible<()> {
    let pow = Expr::Function {
        input: vec![col("a"), col("b")],
        function: FunctionExpr::Pow(PowFunction::Generic),
    };
    assert!(permutation_behavior(&pow.first(), Resize::Allow, None).is_err());
    let array_get = Expr::Function {
        input: vec![col("array"), lit(0i64)],
        function: FunctionExpr::ArrayExpr(ArrayFunction::Get(true)),
    };
    assert!(permutation_behavior(&array_get, Resize::Allow, None).is_err());
    let categories = Expr::Function {
        input: vec![col("category")],
        function: FunctionExpr::Categorical(CategoricalFunction::GetCategories),
    };
    assert!(permutation_behavior(&categories, Resize::Allow, None).is_err());
    Ok(())
}

#[test]
fn test_group_locality_validation() -> Fallible<()> {
    let range = Expr::Literal(LiteralValue::Range(polars_plan::plans::RangeLiteralValue {
        low: 0,
        high: 2,
        dtype: DataType::Int32,
    }));
    assert!(!is_permutation_equivariant(&range));
    assert!(permutation_behavior(&range, Resize::Allow, None).is_err());
    // A genuine scalar list broadcasts, unlike a positional Series literal.
    let list = Expr::Literal(LiteralValue::Scalar(Scalar::new(
        DataType::List(Box::new(DataType::Int32)),
        AnyValue::List(Series::new("list".into(), [1i32, 2])),
    )));
    assert!(is_permutation_equivariant(&list));
    assert!(
        permutation_behavior(
            &lit(Series::new("weights".into(), [1i32, 0])),
            Resize::Allow,
            None
        )
        .is_err()
    );
    assert!(permutation_behavior(&col("x").agg_groups(), Resize::Allow, None).is_err());
    for quantile in [
        col("q").first(),
        lit(-0.1),
        lit(1.1),
        lit(f64::NAN),
        lit(f64::INFINITY),
        lit(NULL),
        lit("0.5"),
        lit(true),
        lit(Series::new("q".into(), [0.5f64])),
    ] {
        assert!(check_quantile(&quantile).is_err(), "{quantile:?}");
    }
    for quantile in [lit(0), lit(1), lit(0.5)] {
        check_quantile(&quantile)?;
    }
    Ok(())
}

#[test]
fn test_permutation_checker_ternary_implicit_casts() -> Fallible<()> {
    let categories = Categories::global();
    let schema = Schema::from_iter([
        Field::new("mask".into(), DataType::Boolean),
        Field::new("i".into(), DataType::Int32),
        Field::new("j".into(), DataType::Int64),
        Field::new("f".into(), DataType::Float64),
        Field::new("text".into(), DataType::String),
        Field::new("date".into(), DataType::Date),
        Field::new(
            "category".into(),
            DataType::Categorical(categories.clone(), categories.mapping()),
        ),
    ]);
    let select = |truthy, falsy| when(col("mask")).then(truthy).otherwise(falsy).n_unique();

    // Equal dtypes, primitive numeric promotions, and null promotion
    // cannot introduce value-dependent strict-cast errors.
    for expr in [
        select(col("i"), col("i")),
        select(col("i"), col("j")),
        select(col("i"), col("f")),
        select(col("i"), lit(NULL)),
    ] {
        check_permutation_invariance(&expr, &schema)?;
    }

    // Polars would insert strict casts for these mixed branch dtypes.
    // A cast may fail depending on the values of private rows.
    for expr in [
        select(col("date"), col("text")),
        select(col("category"), col("text")),
        select(col("i"), col("text")),
        when(col("text"))
            .then(col("i"))
            .otherwise(col("i"))
            .n_unique(),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "unexpectedly accepted: {expr:?}"
        );
    }
    Ok(())
}
