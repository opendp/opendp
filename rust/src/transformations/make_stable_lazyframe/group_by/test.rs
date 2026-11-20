use super::*;
#[cfg(not(feature = "polars-ffi"))]
use crate::core::Domain;
use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

#[test]
fn test_permutation_checker_rejects_data_dependent_errors() -> Fallible<()> {
    // Failures are rejected independently of permutation behavior.
    assert!(inspect_expr(&lit("a").strict_cast(DataType::Int32), Resize::Allow, None).is_err());
    assert!(inspect_expr(&col("t").dt().year(), Resize::Allow, None).is_err());
    assert!(inspect_expr(&col("t").dt().hour(), Resize::Allow, None).is_err());
    Ok(())
}

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
        inspect_expr(&col("A"), Resize::Ban, Some(&schema))?,
        PermutationBehavior::Equivariant
    );
    assert!(is_permutation_equivariant(&col("A")));
    assert!(!is_permutation_equivariant(&(col("A") + lit(1))));
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
        inspect_expr(&col("A").sum(), Resize::Ban, Some(&schema))?,
        PermutationBehavior::Invariant
    );
    // Data-dependent resizes and order-dependent intermediate results are rejected.
    assert!(inspect_expr(&col("A").unique(), Resize::Ban, Some(&schema)).is_err());
    assert!(
        inspect_expr(
            &(col("A").unique() + col("B")),
            Resize::Allow,
            Some(&schema)
        )
        .is_err()
    );
    assert!(inspect_expr(&col("A").shift(lit(1)).sum(), Resize::Allow, Some(&schema)).is_err());
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
        col("i").cast(DataType::Int64).sum(),
        col("i").fill_null(lit(0)).sum(),
        when(col("flag")).then(col("i")).otherwise(lit(0)).sum(),
        col("i").filter(col("flag")).sum(),
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
        Field::new("y".into(), DataType::Float64),
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
            inspect_expr(&expr, Resize::Allow, Some(&schema))?,
            PermutationBehavior::Equivariant,
            "{expr:?}"
        );
    }
    for expr in [col("x").sum(), col("x").min(), col("x").mean()] {
        assert!(check_permutation_invariance(&expr, &schema).is_err());
    }
    // Binary operators require separate infallibility certification.
    for expr in [
        col("x") / col("y"),
        col("x") * lit(2.0),
        col("i") / col("i"),
    ] {
        assert!(inspect_expr(&expr, Resize::Allow, Some(&schema)).is_err());
    }
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_unsafe_parameters() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("i".into(), DataType::Int32),
        Field::new("lower".into(), DataType::Int32),
        Field::new("flag".into(), DataType::Boolean),
        Field::new(
            "category".into(),
            DataType::Categorical(Categories::global(), Categories::global().mapping()),
        ),
    ]);
    for expr in [
        col("i").strict_cast(DataType::Int64).sum(),
        col("i").cast(DataType::String).n_unique(),
        col("category").cast(DataType::UInt32).sum(),
        col("category").to_physical().sum(),
        col("i").clip(col("lower"), lit(10)).sum(),
        col("i").fill_null(col("lower")).sum(),
        col("i").filter(col("i")).sum(),
        col("i").sort(Default::default()).min(),
        col("i").reverse().sum(),
        col("i").shift(lit(1)).sum(),
        col("i").abs().sum(),
        col("i").unique().min(),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "unexpectedly accepted: {expr:?}"
        );
    }
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_enum_string_ordering() -> Fallible<()> {
    let categories = FrozenCategories::new(["low", "high"].into_iter())?;
    let enum_dtype = DataType::Enum(categories.clone(), categories.mapping().clone());
    let schema = Schema::from_iter([
        Field::new("level".into(), enum_dtype),
        Field::new("text".into(), DataType::String),
    ]);

    // An unknown private string causes Polars Enum ordering comparisons to
    // fail at execution time. The invariant reducer must not hide that risk.
    for comparison in [
        col("level").lt(col("text")),
        col("level").lt_eq(col("text")),
        col("level").gt(col("text")),
        col("level").gt_eq(col("text")),
        col("text").lt(col("level")),
        col("text").gt(col("level")),
    ] {
        assert!(
            check_permutation_invariance(&comparison.clone().sum(), &schema).is_err(),
            "unexpectedly accepted: {comparison:?}"
        );
    }
    // All binary operations, including ordinary string comparisons, are rejected.
    assert!(check_permutation_invariance(&col("text").eq(lit("low")).sum(), &schema).is_err());
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_unrelated_functions() -> Fallible<()> {
    let array_get = Expr::Function {
        input: vec![col("array"), lit(0i64)],
        function: FunctionExpr::ArrayExpr(ArrayFunction::Get(true)),
    };
    let list_get = Expr::Function {
        input: vec![col("list"), lit(0i64)],
        function: FunctionExpr::ListExpr(ListFunction::Get(true)),
    };
    let extract = Expr::Function {
        input: vec![col("text"), lit("[a-z]+")],
        function: FunctionExpr::StringExpr(StringFunction::Extract(0)),
    };
    let hashed = Expr::Function {
        input: vec![col("integer")],
        function: FunctionExpr::Hash(1, 2, 3, 4),
    };
    let repeated = Expr::Function {
        input: vec![lit(1i32), col("n")],
        function: FunctionExpr::Repeat,
    };
    for expr in [array_get, list_get, extract, hashed, repeated] {
        assert!(
            inspect_expr(&expr, Resize::Allow, None).is_err(),
            "{expr:?}"
        );
    }
    Ok(())
}

#[test]
fn test_permutation_checker_rejects_dtype_wide_categories() -> Fallible<()> {
    let categories = Expr::Function {
        input: vec![col("category")],
        function: FunctionExpr::Categorical(CategoricalFunction::GetCategories),
    };
    assert!(inspect_expr(&categories, Resize::Allow, None).is_err());
    Ok(())
}

#[test]
fn test_group_by_materializes_computed_margin_keys() -> Fallible<()> {
    let key = col("A") + col("B");
    let lf = df!("A" => [20i32, 18, 16, 14], "B" => [0i32, 1, 2, 3])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        Margin::by([key.clone()])
            .with_max_groups(4)
            .with_max_length(1)
            .with_invariant_keys(),
    )?;
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        FrameDistance(SymmetricDistance),
        lf.clone().group_by([key]).agg([col("B").sum()]),
    )?;

    // Grouping by A + B emits the computed value as column A. Its partition
    // certificate remains valid for A, but not for reevaluating A + B.
    assert_eq!(
        t.output_domain.margins,
        vec![Margin::by([col("A")]).with_max_groups(4).with_max_length(1)]
    );
    #[cfg(not(feature = "polars-ffi"))]
    assert!(t.output_domain.member(&t.invoke(&lf)?)?);
    assert_eq!(t.map(&Bounds::from(1))?, Bounds::from(2));
    Ok(())
}

#[test]
fn test_group_by_materializes_aliased_margin_keys() -> Fallible<()> {
    let key = col("A").alias("key");
    let lf = df!("A" => [1i32, 2])?.lazy();
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?
        .with_margin(Margin::by([key.clone()]).with_max_groups(2))?;
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        FrameDistance(SymmetricDistance),
        lf.group_by([key]).agg([]),
    )?;

    assert_eq!(
        t.output_domain.margins,
        vec![Margin::by([col("key")]).with_max_groups(2)]
    );
    Ok(())
}

#[test]
fn test_group_by_preserves_plain_margin_keys() -> Fallible<()> {
    let lf = df!("A" => [1i32, 2], "B" => [3i32, 4])?.lazy();
    let margin = Margin::by([col("A")])
        .with_max_groups(2)
        .with_max_length(1)
        .with_invariant_lengths();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(margin.clone())?;
    let t: Transformation<_, _, _, FrameDistance<SymmetricDistance>> = make_stable_lazyframe(
        domain,
        FrameDistance(SymmetricDistance),
        lf.group_by([col("A")]).agg([col("B").sum()]),
    )?;

    assert_eq!(
        t.output_domain.margins,
        vec![Margin {
            invariant: None,
            ..margin
        }]
    );
    Ok(())
}

#[test]
fn test_permutation_checker_pow_and_qcut() -> Fallible<()> {
    // Integer pow may fail when a private exponent is negative.
    let pow = Expr::Function {
        input: vec![col("a"), col("b")],
        function: FunctionExpr::Pow(PowFunction::Generic),
    };
    assert!(inspect_expr(&pow.first(), Resize::Allow, None).is_err());

    // Even infallible operations remain unsupported until certified.
    let sqrt = Expr::Function {
        input: vec![col("a")],
        function: FunctionExpr::Pow(PowFunction::Sqrt),
    };
    assert!(inspect_expr(&sqrt, Resize::Allow, None).is_err());

    for allow_duplicates in [false, true] {
        let qcut = Expr::Function {
            input: vec![col("v")],
            function: FunctionExpr::QCut {
                probs: vec![0.25, 0.5, 0.75],
                labels: None,
                left_closed: false,
                allow_duplicates,
                include_breaks: false,
            },
        };
        assert!(inspect_expr(&qcut.first(), Resize::Allow, None).is_err());
    }

    Ok(())
}

#[test]
fn test_stable_group_by_categorical() -> Fallible<()> {
    use crate::domains::{CategoricalDomain, LazyFrameDomain};
    use crate::metrics::SymmetricDistance;
    use crate::transformations::make_stable_lazyframe;

    let categories = Categories::global();
    let dtype = DataType::Categorical(categories.clone(), categories.mapping());
    let series = Series::new("category".into(), ["B", "A", "B"]).cast(&dtype)?;
    let lf = DataFrame::new(vec![series.into_column()])?.lazy();
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "category",
        CategoricalDomain::default(),
    )])?;

    let t = make_stable_lazyframe(
        domain,
        FrameDistance(SymmetricDistance),
        lf.clone().group_by(["category"]).agg([len()]),
    )?;

    assert_eq!(t.invoke(&lf)?.collect()?.height(), 2);
    Ok(())
}

#[test]
fn test_permutation_invariance_validation() -> Fallible<()> {
    let schema = Schema::from_iter([
        Field::new("i".into(), DataType::Int32),
        Field::new("x".into(), DataType::Float64),
    ]);
    for expr in [col("i").sum(), col("i").min(), col("i").max()] {
        check_permutation_invariance(&expr, &schema)?;
    }
    for expr in [
        col("i").clip(lit(0), lit(10)).min(),
        col("i").is_null().sum(),
        col("i").fill_null(lit(0)).sum(),
        col("i").drop_nulls().min(),
    ] {
        check_permutation_invariance(&expr, &schema)?;
    }
    for expr in [
        col("i").shift(lit(1)).sum(),
        col("i").reverse().min(),
        col("i").sum() + lit(1),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "{expr:?}"
        );
    }
    for expr in [
        col("i").sort(Default::default()).first(),
        col("i").sort(Default::default()).mean(),
        col("i").sort(Default::default()).sum(),
        col("i")
            .sort(Default::default())
            .quantile(lit(0.5), QuantileMethod::Linear),
        col("x").sum(),
        col("x").mean(),
        col("x").min(),
        col("x").max(),
        col("x").first(),
        col("x").sort(Default::default()).first(),
        col("x").sort(Default::default()).mean(),
        (col("i") * lit(Series::new("weights".into(), [1i32, 0]))).sum(),
        col("x").sort(Default::default()).agg_groups(),
        col("x")
            .sort(Default::default())
            .quantile(col("x").first(), QuantileMethod::Linear),
    ] {
        assert!(
            check_permutation_invariance(&expr, &schema).is_err(),
            "{expr:?}"
        );
    }
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
    assert!(inspect_expr(&range, Resize::Allow, None).is_err());
    // A genuine scalar list broadcasts, unlike a positional Series literal.
    let list = Expr::Literal(LiteralValue::Scalar(Scalar::new(
        DataType::List(Box::new(DataType::Int32)),
        AnyValue::List(Series::new("list".into(), [1i32, 2])),
    )));
    assert!(is_permutation_equivariant(&list));
    inspect_expr(&list, Resize::Allow, None)?;
    assert!(
        inspect_expr(
            &(col("i") * lit(Series::new("weights".into(), [1i32, 0]))).sum(),
            Resize::Allow,
            None
        )
        .is_err()
    );
    assert!(
        inspect_expr(
            &col("x").sort(Default::default()).agg_groups(),
            Resize::Allow,
            None
        )
        .is_err()
    );
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
        assert!(
            inspect_expr(
                &col("x").quantile(quantile, QuantileMethod::Linear),
                Resize::Allow,
                None
            )
            .is_err()
        );
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
