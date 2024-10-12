use super::*;
use crate::domains::AtomDomain;

#[test]
fn test_frame_new() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<f64>::default()),
    ])?;

    let lf = df!("A" => &[3, 4, 5], "B" => &[1., 3., 7.])?.lazy();

    assert!(lf_domain.member(&lf)?);

    Ok(())
}

#[test]
fn test_margin() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<String>::default()),
    ])?
    .with_margin(
        &["A"],
        Margin::default()
            .with_max_partition_length(1)
            .with_max_num_partitions(2),
    )?;

    let lf_exceed_partition_size = df!("A" => [1, 2, 2], "B" => ["1", "1", "2"])?.lazyframe();
    assert!(!lf_domain.member(&lf_exceed_partition_size)?);

    let lf_exceed_num_partitions = df!("A" => [1, 2, 3], "B" => ["1", "1", "1"])?.lazyframe();
    assert!(!lf_domain.member(&lf_exceed_num_partitions)?);

    let lf = df!("A" => [1, 2], "B" => ["1", "1"])?.lazyframe();
    assert!(lf_domain.member(&lf)?);

    Ok(())
}

#[test]
fn test_get_margin_max_partition_descriptors() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        &["A"],
        Margin::default()
            .with_max_partition_length(10)
            .with_max_partition_contributions(3),
    )?;

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["A".to_string(), "B".to_string()]))
            .max_partition_length,
        Some(10)
    );

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["B".to_string()]))
            .max_partition_length,
        None
    );

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["A".to_string(), "B".to_string()]))
            .max_partition_contributions,
        Some(3)
    );

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["B".to_string()]))
            .max_partition_contributions,
        None
    );
    Ok(())
}

#[test]
fn test_get_margin_covering_small_to_large() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        &["A"],
        Margin::default()
            .with_max_num_partitions(10)
            .with_max_influenced_partitions(3),
    )?
    .with_margin(
        &["B"],
        Margin::default()
            .with_max_num_partitions(10)
            .with_max_influenced_partitions(3),
    )?;

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["A".to_string(), "B".to_string()]))
            .max_num_partitions,
        Some(100)
    );

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["B".to_string()]))
            .max_influenced_partitions,
        Some(3)
    );
    Ok(())
}

#[test]
fn test_get_margin_covering_large_to_small() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        &["A", "B"],
        Margin::default()
            .with_max_num_partitions(10)
            .with_max_influenced_partitions(3),
    )?;

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["A".to_string()]))
            .max_num_partitions,
        Some(10)
    );

    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["B".to_string()]))
            .max_influenced_partitions,
        Some(3)
    );
    Ok(())
}

#[test]
fn test_get_margin_public_info() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(&["A", "B"], Margin::default().with_public_keys())?;

    // keys are known on coarser partitions
    assert_eq!(
        lf_domain
            .get_margin(BTreeSet::from(["A".to_string()]))
            .public_info,
        Some(MarginPub::Keys)
    );
    Ok(())
}

#[test]
fn test_option_or() {
    assert_eq!(option_min(Some(1), Some(2)), Some(1));
    assert_eq!(option_min(Some(1), None), Some(1));
    assert_eq!(option_min(None, Some(1)), Some(1));
    assert_eq!(option_min::<i32>(None, None), None);
}
