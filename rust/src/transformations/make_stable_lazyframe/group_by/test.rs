use super::*;
use crate::core::Domain;
use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

#[test]
fn test_check_infallible() -> Fallible<()> {
    // failure to cast causes a data-dependent error
    assert!(check_infallible(&lit("a").strict_cast(DataType::Int32), Resize::Allow).is_err());

    Ok(())
}

#[test]
fn test_check_infallible_resize() -> Fallible<()> {
    // col doesn't resize, so passes the ban
    assert!(check_infallible(&col("A"), Resize::Ban).is_ok());
    // sum results in a broadcastable scalar, so it passes the ban
    assert!(check_infallible(&col("A").sum(), Resize::Ban).is_ok());
    // unique resizes, so fails the ban
    assert!(check_infallible(&col("A").unique(), Resize::Ban).is_err());
    // resizing behind an aggregation is allowed, though, because it can broadcast
    assert!(check_infallible(&col("A").unique().sum(), Resize::Ban).is_ok());
    // while resizing is generally allowed, binary ops ban resizing
    assert!(check_infallible(&(col("A").unique() + col("B")), Resize::Allow).is_err());
    // the sum results in a broadcastable scalar, so it passes the binary op resize ban
    assert!(check_infallible(&(col("A").unique().sum() + col("B")), Resize::Ban).is_ok());

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
