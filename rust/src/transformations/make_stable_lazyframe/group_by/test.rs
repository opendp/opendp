use super::*;
#[cfg(not(feature = "polars-ffi"))]
use crate::core::Domain;
use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

#[test]
fn test_group_by_output_margins() -> Fallible<()> {
    for (key, name) in [
        (col("A"), "A"),
        (col("A").alias("key"), "key"),
        (col("A") + col("B"), "A"),
    ] {
        let domain = DslPlanDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<i32>::default()),
        ])?
        .with_margin(
            Margin::by([key.clone()])
                .with_max_groups(4)
                .with_max_length(3)
                .with_invariant_lengths(),
        )?
        .with_margin(Margin::by([col("B")]).with_max_groups(8))?;
        let output = group_by_output_domain(
            &domain,
            &HashSet::from([key.clone()]),
            &[key],
            &[col("B").sum()],
        )?;
        assert_eq!(
            output.margins,
            vec![
                Margin::by([col(name)])
                    .with_max_groups(4)
                    .with_max_length(3)
            ]
        );
    }
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

    let actual = t.invoke(&lf)?.collect()?;
    let labels = actual.column("category")?.cast(&DataType::String)?;
    let counts = actual.column("len")?.u32()?;
    let groups: std::collections::HashMap<_, _> = labels
        .str()?
        .into_no_null_iter()
        .zip(counts.into_no_null_iter())
        .collect();
    assert_eq!(
        groups,
        std::collections::HashMap::from([("A", 1), ("B", 2)])
    );
    Ok(())
}
