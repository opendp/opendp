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

fn assert_row_descriptors<F: Frame>(
    domain: &FrameDomain<F>,
    by: &[&str],
    max_partition_length: Option<u32>,
    max_partition_contributions: Option<u32>,
) {
    let margin = domain.get_margin(&by.iter().map(|s| (*s).into()).collect());
    assert_eq!(margin.max_partition_length, max_partition_length);
    assert_eq!(
        margin.max_partition_contributions,
        max_partition_contributions
    );
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

    assert_row_descriptors(&lf_domain, &["A", "B"], Some(10), Some(3));
    assert_row_descriptors(&lf_domain, &["B"], None, None);
    Ok(())
}

fn assert_partition_descriptors<F: Frame>(
    domain: &FrameDomain<F>,
    by: &[&str],
    max_num_partitions: Option<u32>,
    max_influenced_partitions: Option<u32>,
) {
    let margin = domain.get_margin(&by.iter().map(|s| (*s).into()).collect());
    assert_eq!(margin.max_num_partitions, max_num_partitions);
    assert_eq!(margin.max_influenced_partitions, max_influenced_partitions);
}

#[test]
fn test_get_margin_covering_small_to_large() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
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
            .with_max_num_partitions(11)
            .with_max_influenced_partitions(4),
    )?;

    assert_partition_descriptors(&lf_domain, &["A", "B"], Some(110), Some(12));
    assert_partition_descriptors(&lf_domain, &["B"], Some(11), Some(4));
    assert_partition_descriptors(&lf_domain, &[], Some(1), Some(1));
    assert_partition_descriptors(&lf_domain, &["C"], None, None);
    Ok(())
}

#[test]
fn test_get_margin_covering_large_to_small() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(
        &["A", "B"],
        Margin::default()
            .with_max_num_partitions(10)
            .with_max_influenced_partitions(3),
    )?;

    assert_partition_descriptors(&lf_domain, &["A"], Some(10), Some(3));
    assert_partition_descriptors(&lf_domain, &["B"], Some(10), Some(3));
    assert_partition_descriptors(&lf_domain, &[], Some(1), Some(1));
    assert_partition_descriptors(&lf_domain, &["C"], None, None);
    Ok(())
}

#[test]
fn test_get_margin_public_info() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(&["A", "B"], Margin::default().with_public_lengths())?;

    // nothing is known when grouping not in margins
    let margin_abc = lf_domain.get_margin(&BTreeSet::from(["A".into(), "B".into(), "C".into()]));
    assert_eq!(margin_abc.public_info, None);

    // retrieving info directly from the margin as-is
    let margin_ab = lf_domain.get_margin(&BTreeSet::from(["A".into(), "B".into()]));
    assert_eq!(margin_ab.public_info, Some(MarginPub::Lengths));

    // keys and lengths are known on coarser partitions
    let margin_a = lf_domain.get_margin(&BTreeSet::from(["A".into()]));
    assert_eq!(margin_a.public_info, Some(MarginPub::Lengths));
    Ok(())
}

#[test]
fn test_find_min_covering_optimal() -> Fallible<()> {
    let must_cover = BTreeSet::from([1u32, 2, 3, 4, 5]);
    let sets = [
        BTreeSet::from([1, 2, 3]),
        BTreeSet::from([2, 4]),
        BTreeSet::from([3, 4]),
        BTreeSet::from([4, 5]),
    ];
    let covering =
        find_min_covering(must_cover.clone(), sets.iter().map(|k| (k, 1)).collect()).unwrap();

    assert_eq!(covering.len(), 2);

    let intersection = covering
        .into_iter()
        .map(|(k, _)| k.clone())
        .reduce(|l, r| &l | &r)
        .unwrap();
    assert_eq!(intersection, must_cover);
    Ok(())
}

#[test]
fn test_find_min_covering_nonoptimal() -> Fallible<()> {
    let must_cover = BTreeSet::from_iter(1..=14);

    // optimal covering is the first two sets,
    // but the greedy algorithm non-optimally chooses the last three sets
    let sets = [
        BTreeSet::from_iter(1..=7),
        BTreeSet::from_iter(8..=14),
        BTreeSet::from([1, 8]),
        BTreeSet::from([2, 3, 9, 10]),
        BTreeSet::from([4, 5, 6, 7, 11, 12, 13, 14]),
    ];
    let covering =
        find_min_covering(must_cover.clone(), sets.iter().map(|k| (k, 1)).collect()).unwrap();

    assert_eq!(covering.len(), 3);

    let intersection = covering
        .into_iter()
        .map(|(k, _)| k.clone())
        .reduce(|l, r| &l | &r)
        .unwrap();
    assert_eq!(intersection, must_cover);
    Ok(())
}
