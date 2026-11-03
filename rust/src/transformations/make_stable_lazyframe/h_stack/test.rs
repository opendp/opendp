use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::{FrameDistance, SymmetricDistance},
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_with_column() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?
    .with_margin(Margin::by(["chunk_2_null"]).with_invariant_keys())?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().with_column(lit(2).gt(col("chunk_2_null"))),
    )?;

    let actual = t_with_column.invoke(&lf)?.collect()?;
    let expect = df!(
        "chunk_2_null" => [Some(1), None],
        "literal" => [Some(true), None]
    )?;
    assert_eq!(actual, expect);
    // while margins get filtered out, chunk_2_null should still be present
    assert_eq!(
        t_with_column.output_domain.margins,
        t_with_column.input_domain.margins
    );

    Ok(())
}

// LazyFrame::fill_nan works for "free", because it breaks down to pre-existing primitives:
//    hstack with expressions that use fill_nan and alias
#[test]
// TODO: ignored because Polars 1.0.0 changes behavior of fill_nan to de-sugar into .with_columns(all().fill_nan(v)),
// and we don't support all() at this time. See https://github.com/opendp/opendp/issues/1772
#[ignore]
fn test_fill_nan() -> Fallible<()> {
    let lf = df!("f64" => [1f64, f64::NAN])?.lazy();

    let series_domain = SeriesDomain::new("f64", AtomDomain::<f64>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?;

    let t_with_column = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().fill_nan(lit(2.)),
    )?;

    let actual = t_with_column.invoke(&lf)?.collect()?;
    let expect = df!("f64" => [1f64, 2f64])?;

    assert_eq!(actual, expect);
    Ok(())
}

#[test]
fn test_with_columns_preserve_margin_lineage() -> Fallible<()> {
    let lf = df!("k" => [0i32, 0], "x" => [0i32, 1])?.lazy();
    let computed_key = col("k").cast(DataType::Int64);

    for margin in [
        Margin::by([computed_key.clone()]).with_invariant_keys(),
        Margin::by([computed_key.clone()])
            .with_invariant_lengths()
            .with_max_length(2)
            .with_max_groups(1),
        Margin::by([computed_key.clone()])
            .with_max_length(2)
            .with_max_groups(1),
    ] {
        let unrelated = Margin::by(["x"]).with_invariant_keys();
        let frame = Margin::select().with_max_length(2);
        let domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("k", AtomDomain::<i32>::default()),
            SeriesDomain::new("x", AtomDomain::<i32>::default()),
        ])?
        .with_margin(margin.clone())?
        .with_margin(unrelated.clone())?
        .with_margin(frame.clone())?;

        let preserved = vec![margin.clone(), unrelated.clone(), frame.clone()];
        for (column, expected_margins) in [
            (col("k").alias("k"), preserved.clone()),
            (col("k"), preserved.clone()),
            (lit(1).alias("new"), preserved),
            (col("x").alias("k"), vec![unrelated.clone(), frame.clone()]),
            (col("k").alias("x"), vec![margin.clone(), frame.clone()]),
        ] {
            let t = make_stable_lazyframe(
                domain.clone(),
                FrameDistance(SymmetricDistance),
                lf.clone().with_column(column),
            )?;
            assert_eq!(t.output_domain.margins, expected_margins);
        }
    }
    Ok(())
}

#[test]
fn test_with_columns_invalidate_contribution_bounds() -> Fallible<()> {
    use crate::metrics::Bound;

    let lf = df!("k" => [0i32, 1], "x" => [0i32, 1])?.lazy();
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("k", AtomDomain::<i32>::default()),
        SeriesDomain::new("x", AtomDomain::<i32>::default()),
    ])?;

    let by_k = Bound::by(["k"]).with_per_group(1).with_num_groups(2);
    let by_computed_k = Bound::by([col("k").cast(DataType::Int64)])
        .with_per_group(1)
        .with_num_groups(2);
    let by_x = Bound::by(["x"]).with_per_group(2).with_num_groups(2);

    let input = Bounds::from(10)
        .with_bound(by_k.clone())
        .with_bound(by_computed_k.clone())
        .with_bound(by_x.clone());
    let all = input.clone();
    let without_k = Bounds::from(10).with_bound(by_x.clone());
    let without_x = Bounds::from(10).with_bound(by_k).with_bound(by_computed_k);

    for (expr, expected) in [
        (col("k").alias("k"), all.clone()),
        (col("x").alias("k"), without_k.clone()),
        (lit(0).alias("k"), without_k),
        (col("k").alias("x"), without_x),
        (lit(0).alias("new"), all),
    ] {
        let t = make_stable_lazyframe(
            domain.clone(),
            FrameDistance(SymmetricDistance),
            lf.clone().with_column(expr),
        )?;
        assert_eq!(t.map(&input)?, expected);
    }
    Ok(())
}

#[test]
fn test_with_columns_invalidate_selector_grouping() -> Fallible<()> {
    use crate::metrics::Bound;

    let lf = df!("k" => [0i32, 1], "x" => [0i32, 0])?.lazy();
    for key in [
        cols(["k", "x"]).as_expr(),
        Selector::ByDType(dtype_cols([DataType::Int32])).as_expr(),
    ] {
        assert!(key.clone().meta().root_names().is_empty());
        assert!(key.clone().meta().has_multiple_outputs());

        let margin = Margin::by([key.clone()]).with_max_length(1);
        let domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("k", AtomDomain::<i32>::default()),
            SeriesDomain::new("x", AtomDomain::<i32>::default()),
        ])?
        .with_margin(margin.clone())?;
        let input =
            Bounds::from(2).with_bound(Bound::by([key]).with_per_group(1).with_num_groups(2));

        for (expr, preserve) in [
            (col("k").alias("k"), true),
            (lit(0).alias("k"), false),
            (lit(0).alias("new"), false),
        ] {
            let t = make_stable_lazyframe(
                domain.clone(),
                FrameDistance(SymmetricDistance),
                lf.clone().with_column(expr),
            )?;
            let expected_margins = if preserve {
                vec![margin.clone()]
            } else {
                vec![]
            };
            assert_eq!(t.output_domain.margins, expected_margins);
            let expected_bounds = if preserve {
                input.clone()
            } else {
                Bounds::from(2)
            };
            assert_eq!(t.map(&input)?, expected_bounds);
        }
    }
    Ok(())
}
