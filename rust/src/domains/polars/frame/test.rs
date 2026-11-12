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
    .with_margin(Margin::by(["A"]).with_max_length(1).with_max_groups(2))?;

    let lf_exceed_partition_size = df!("A" => [1, 2, 2], "B" => ["1", "1", "2"])?.lazy();
    assert!(!lf_domain.member(&lf_exceed_partition_size)?);

    let lf_exceed_num_partitions = df!("A" => [1, 2, 3], "B" => ["1", "1", "1"])?.lazy();
    assert!(!lf_domain.member(&lf_exceed_num_partitions)?);

    let lf = df!("A" => [1, 2], "B" => ["1", "1"])?.lazy();
    assert!(lf_domain.member(&lf)?);

    Ok(())
}

#[test]
fn test_margin_select() -> Fallible<()> {
    let lf_domain =
        LazyFrameDomain::new(vec![SeriesDomain::new("A", AtomDomain::<i32>::default())])?;
    let lf = df!("A" => [1, 2, 3])?.lazy();

    for (max_length, expected) in [(2, false), (3, true)] {
        let domain = lf_domain
            .clone()
            .with_margin(Margin::select().with_max_length(max_length))?;
        assert_eq!(domain.member(&lf)?, expected);
    }

    let zero_groups = lf_domain
        .clone()
        .with_margin(Margin::select().with_max_groups(0))?;
    assert!(!zero_groups.member(&lf)?);

    let empty = lf.filter(lit(false));
    assert!(!zero_groups.member(&empty)?);
    assert!(
        lf_domain
            .clone()
            .with_margin(Margin::select().with_max_length(0))?
            .member(&empty)?
    );
    assert!(
        lf_domain
            .with_margin(Margin::by(["A"]).with_max_length(0).with_max_groups(0))?
            .member(&empty)?
    );

    Ok(())
}

fn assert_row_descriptors<F: Frame<Expr = Expr>>(domain: &FrameDomain<F>, by: &[&str], max_length: Option<u32>) {
    let margin = domain.get_margin(&by.iter().map(|s| (*s).into()).collect());
    assert_eq!(margin.max_length, max_length);
}

#[test]
fn test_get_margin_max_partition_descriptors() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_length(10))?;

    assert_row_descriptors(&lf_domain, &["A", "B"], Some(10));
    assert_row_descriptors(&lf_domain, &["B"], None);
    Ok(())
}

fn assert_group_descriptors<F: Frame<Expr = Expr>>(
    domain: &FrameDomain<F>,
    by: &[&str],
    max_groups: Option<u32>,
) {
    let margin = domain.get_margin(&by.iter().map(|s| (*s).into()).collect());
    assert_eq!(margin.max_groups, max_groups);
}

#[test]
fn test_get_margin_covering_small_to_large() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A"]).with_max_groups(10))?
    .with_margin(Margin::by(["B"]).with_max_groups(11))?;

    assert_group_descriptors(&lf_domain, &["A", "B"], Some(110));
    assert_group_descriptors(&lf_domain, &["B"], Some(11));
    assert_group_descriptors(&lf_domain, &[], Some(1));
    assert_group_descriptors(&lf_domain, &["C"], None);
    Ok(())
}

#[test]
fn test_get_margin_covering_large_to_small() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
        SeriesDomain::new("C", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A", "B"]).with_max_groups(10))?;

    assert_group_descriptors(&lf_domain, &["A"], Some(10));
    assert_group_descriptors(&lf_domain, &["B"], Some(10));
    assert_group_descriptors(&lf_domain, &[], Some(1));
    assert_group_descriptors(&lf_domain, &["C"], None);
    Ok(())
}

#[test]
fn test_get_margin_invariant() -> Fallible<()> {
    let lf_domain = LazyFrameDomain::new(vec![
        SeriesDomain::new("A", AtomDomain::<i32>::default()),
        SeriesDomain::new("B", AtomDomain::<i32>::default()),
    ])?
    .with_margin(Margin::by(["A", "B"]).with_invariant_lengths())?;

    // nothing is known when grouping not in margins
    let margin_abc = lf_domain.get_margin(&HashSet::from(["A".into(), "B".into(), "C".into()]));
    assert_eq!(margin_abc.invariant, None);

    // retrieving info directly from the margin as-is
    let margin_ab = lf_domain.get_margin(&HashSet::from(["A".into(), "B".into()]));
    assert_eq!(margin_ab.invariant, Some(Invariant::Lengths));

    // keys and lengths are known on coarser partitions
    let margin_a = lf_domain.get_margin(&HashSet::from(["A".into()]));
    assert_eq!(margin_a.invariant, Some(Invariant::Lengths));
    Ok(())
}

#[test]
fn test_find_min_covering_optimal() -> Fallible<()> {
    let must_cover = HashSet::from([1u32, 2, 3, 4, 5]);
    let sets = [
        HashSet::from([1, 2, 3]),
        HashSet::from([2, 4]),
        HashSet::from([3, 4]),
        HashSet::from([4, 5]),
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
    let must_cover = HashSet::from_iter(1..=14);

    // optimal covering is the first two sets,
    // but the greedy algorithm non-optimally chooses the last three sets
    let sets = [
        HashSet::from_iter(1..=7),
        HashSet::from_iter(8..=14),
        HashSet::from([1, 8]),
        HashSet::from([2, 3, 9, 10]),
        HashSet::from([4, 5, 6, 7, 11, 12, 13, 14]),
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

#[test]
fn generic_margin_expression_keys() {
    let margin = Margin::<u8>::by([1u8]).with_max_length(3).with_max_groups(4);
    assert!(margin.by.contains(&1));
    assert_eq!(margin.l_0(10), 4);
    assert_eq!(margin.l_inf(10), 3);
}

#[derive(Clone)]
struct SymbolicFixture;

impl Frame for SymbolicFixture {
    type Column = ();
    type Expr = u8;

    fn lazyframe(self) -> Fallible<LazyFrame> {
        fallible!(FailedFunction, "no Polars conversion")
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        fallible!(FailedFunction, "no Polars conversion")
    }
    fn root_names(expr: &u8, columns: &[ColumnDomain<()>]) -> Fallible<Vec<PlSmallStr>> {
        columns.get(*expr as usize)
            .map(|column| vec![column.name.clone()])
            .ok_or_else(|| err!(MakeDomain, "column index outside schema"))
    }
    fn frame_member(&self, _domain: &FrameDomain<Self>) -> Fallible<bool> {
        fallible!(FailedFunction, "no bound input for symbolic plan")
    }
}

#[test]
fn symbolic_frame_shares_margin_inference() -> Fallible<()> {
    let domain = FrameDomain::<SymbolicFixture>::new(vec![
        ColumnDomain::<()>::new("x", AtomDomain::<i64>::default()),
    ])?;
    assert!(domain.clone().with_margin(Margin::<u8>::by([1u8])).is_err());
    let domain = domain.with_margin(Margin::<u8>::by([0u8]).with_max_length(3))?;
    assert_eq!(domain.get_margin(&HashSet::from([0u8])).max_length, Some(3));
    assert_eq!(domain.get_margin(&HashSet::new()).max_length, None);
    assert!(domain.member(&SymbolicFixture).is_err());
    Ok(())
}
