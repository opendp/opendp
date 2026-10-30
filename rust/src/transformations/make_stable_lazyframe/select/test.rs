use crate::{
    domains::{AtomDomain, LazyFrameDomain, Margin, OptionDomain, SeriesDomain},
    metrics::{Bound, Bounds, SymmetricDistance},
    transformations::make_stable_lazyframe,
};

use super::*;

#[test]
fn test_select_microdata() -> Fallible<()> {
    let lf = df!("chunk_2_null" => [Some(1i64), None])?.lazy();

    let lf_domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "chunk_2_null",
        OptionDomain::new(AtomDomain::<i64>::default()),
    )])?;

    let t_select = make_stable_lazyframe(
        lf_domain.clone(),
        FrameDistance(SymmetricDistance),
        lf.clone().select([lit(2).gt(col("chunk_2_null"))]),
    )?;

    let observed = t_select.invoke(&lf)?.collect()?;
    let expected = df!(
        "chunk_2_null" => [Some(1), None],
    )?;
    assert_eq!(observed, expected);
    assert!(t_select.output_domain.margins.is_empty());

    Ok(())
}

#[test]
fn test_select_grouping_lineage() -> Fallible<()> {
    let lf = df!("k" => [0i32, 1], "x" => [1i32, 2])?.lazy();
    let key = col("k").cast(DataType::Int64);
    let margin_k = Margin::by([key.clone()]).with_invariant_keys();
    let margin_x = Margin::by(["x"]).with_max_length(2);
    let frame = Margin::select().with_max_length(2);
    let domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("k", AtomDomain::<i32>::default()),
        SeriesDomain::new("x", AtomDomain::<i32>::default()),
    ])?
    .with_margin(margin_k.clone())?
    .with_margin(margin_x.clone())?
    .with_margin(frame.clone())?;

    let bound_k = Bound::by([key]).with_per_group(1).with_num_groups(2);
    let bound_x = Bound::by(["x"]).with_per_group(2).with_num_groups(2);
    let input = Bounds::from(2)
        .with_bound(bound_k.clone())
        .with_bound(bound_x.clone());

    for (exprs, margins, bounds) in [
        (
            vec![col("k"), col("x")],
            vec![margin_k.clone(), margin_x.clone(), frame.clone()],
            input.clone(),
        ),
        (
            vec![col("k"), lit(1).alias("x")],
            vec![margin_k.clone(), frame.clone()],
            Bounds::from(2).with_bound(bound_k.clone()),
        ),
        (
            vec![col("x")],
            vec![margin_x.clone(), frame.clone()],
            Bounds::from(2).with_bound(bound_x.clone()),
        ),
        (
            vec![col("k").alias("renamed"), col("x")],
            vec![margin_x.clone(), frame.clone()],
            Bounds::from(2).with_bound(bound_x.clone()),
        ),
        (
            vec![col("k").cast(DataType::Int64).alias("k"), col("x")],
            vec![margin_x.clone(), frame.clone()],
            Bounds::from(2).with_bound(bound_x),
        ),
    ] {
        let t = make_stable_lazyframe(
            domain.clone(),
            FrameDistance(SymmetricDistance),
            lf.clone().select(exprs),
        )?;
        assert_eq!(t.output_domain.margins, margins);
        assert_eq!(t.map(&input)?, bounds);
    }
    Ok(())
}

#[test]
fn test_select_selector_grouping() -> Fallible<()> {
    let lf = df!("k" => [0i32, 1], "x" => [1i32, 2])?.lazy();

    for key in [
        cols(["k", "x"]).as_expr(),
        Selector::ByDType(dtype_cols([DataType::Int32])).as_expr(),
    ] {
        let margin = Margin::by([key.clone()]).with_max_length(1);
        let domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("k", AtomDomain::<i32>::default()),
            SeriesDomain::new("x", AtomDomain::<i32>::default()),
        ])?
        .with_margin(margin.clone())?;

        let input =
            Bounds::from(2).with_bound(Bound::by([key]).with_per_group(1).with_num_groups(2));

        for (exprs, unchanged) in [(vec![col("k"), col("x")], true), (vec![col("k")], false)] {
            let t = make_stable_lazyframe(
                domain.clone(),
                FrameDistance(SymmetricDistance),
                lf.clone().select(exprs),
            )?;
            assert_eq!(
                t.output_domain.margins,
                if unchanged {
                    vec![margin.clone()]
                } else {
                    vec![]
                }
            );
            assert_eq!(
                t.map(&input)?,
                if unchanged {
                    input.clone()
                } else {
                    Bounds::from(2)
                }
            );
        }
    }
    Ok(())
}

#[test]
fn test_select_reject_literal_only() -> Fallible<()> {
    let lf = df!("k" => [0i32, 1])?.lazy();
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new("k", AtomDomain::<i32>::default())])?
        .with_margin(Margin::select().with_max_length(2))?;
    for expr in [
        lit(1).alias("constant"),
        (lit(1) + lit(2)).alias("constant"),
    ] {
        assert!(
            make_stable_lazyframe(
                domain.clone(),
                FrameDistance(SymmetricDistance),
                lf.clone().select([expr]),
            )
            .is_err()
        );
    }
    Ok(())
}
