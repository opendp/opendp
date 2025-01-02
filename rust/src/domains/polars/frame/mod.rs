use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use polars::lazy::dsl::{col, len};
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::{LInfDistance, LpDistance};
use crate::traits::{InfMul, ProductOrd};
use crate::{
    core::MetricSpace, domains::SeriesDomain, error::Fallible, transformations::DatasetMetric,
};

use super::NumericDataType;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

pub trait Frame: Clone + Send + Sync {
    /// # Proof Definition
    /// Returns a [`LazyFrame`] containing the same data as in `self`.
    fn lazyframe(self) -> LazyFrame;
    /// # Proof Definition
    /// Returns a [`DataFrame`] containing the same data as in `self`.
    fn dataframe(self) -> Fallible<DataFrame>;
}
impl Frame for LazyFrame {
    fn lazyframe(self) -> LazyFrame {
        self
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        self.collect().map_err(Into::into)
    }
}
impl Frame for DslPlan {
    fn lazyframe(self) -> LazyFrame {
        LazyFrame::from(self)
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        self.lazyframe().collect().map_err(Into::into)
    }
}
impl Frame for DataFrame {
    fn lazyframe(self) -> LazyFrame {
        IntoLazy::lazy(self)
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        Ok(self)
    }
}

/// # Proof Definition
/// `FrameDomain<F>` is the domain of all values of type `F`.
/// * `series_domains` - Vector of Series domains corresponding to each column.
/// * `margins` - Hash map of public information on data stored in the Frame.
///
/// `LazyFrameDomain` is a `FrameDomain<LazyFrame>` and represents all values of type `LazyFrame`.
///
/// ## Generics
/// * `F` - Frame type: DataFrame or LazyFrame.
///
/// # Example
/// ```
/// use polars::prelude::*;
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, Frame, Margin, MarginPub::*};
///
/// // Create a LazyFrameDomain
/// let lf_domain = LazyFrameDomain::new(vec![
///     SeriesDomain::new("A", AtomDomain::<i32>::default()),
///     SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// // Create a LazyFrameDomain with Margin descriptors
/// let lf_domain_with_margins = LazyFrameDomain::new(vec![
///     SeriesDomain::new("A", AtomDomain::<i32>::default()),
///     SeriesDomain::new("B", AtomDomain::<String>::default()),
/// ])?
///         .with_margin(&["A"], Margin::default().with_public_keys())?
///         .with_margin(&["B"], Margin::default().with_public_lengths())?;
///
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone)]
pub struct FrameDomain<F: Frame> {
    pub series_domains: Vec<SeriesDomain>,
    pub margins: HashMap<BTreeSet<PlSmallStr>, Margin>,
    _marker: PhantomData<F>,
}

// manually implemented because F doesn't need PartialEq for FrameDomain to implement PartialEq
impl<F: Frame> PartialEq for FrameDomain<F> {
    fn eq(&self, other: &Self) -> bool {
        self.series_domains == other.series_domains && self.margins == other.margins
    }
}

pub type LazyFrameDomain = FrameDomain<LazyFrame>;
pub(crate) type DslPlanDomain = FrameDomain<DslPlan>;

impl<F: Frame, M: DatasetMetric> MetricSpace for (FrameDomain<F>, M) {
    fn check_space(&self) -> Fallible<()> {
        if M::SIZED
            && self
                .0
                .margins
                .values()
                .all(|m| m.public_info != Some(MarginPub::Lengths))
        {
            return fallible!(MetricSpace, "bounded dataset metric must have known size");
        }
        Ok(())
    }
}

impl<F: Frame, const P: usize, T: ProductOrd + NumericDataType> MetricSpace
    for (FrameDomain<F>, LpDistance<P, T>)
{
    fn check_space(&self) -> Fallible<()> {
        if self
            .0
            .series_domains
            .iter()
            .any(|s| s.dtype() != T::dtype())
        {
            return fallible!(
                MetricSpace,
                "LpDistance requires columns of type {}",
                T::dtype()
            );
        }
        Ok(())
    }
}

impl<Q> MetricSpace for (LazyFrameDomain, LInfDistance<Q>) {
    fn check_space(&self) -> Fallible<()> {
        Ok(())
    }
}

impl<F: Frame> FrameDomain<F> {
    /// Create a new FrameDomain.
    /// Series names must be unique.
    ///
    /// # Proof Definition
    /// Either returns a FrameDomain spanning all dataframes whose columns
    /// are members of `series_domains`, respectively, or an error.
    pub fn new(series_domains: Vec<SeriesDomain>) -> Fallible<Self> {
        Self::new_with_margins(series_domains, HashMap::new())
    }

    /// Create a new FrameDomain.
    /// Series names must be unique.
    ///
    /// # Proof Definition
    /// Either returns a FrameDomain spanning all dataframes whose columns
    /// are members of `series_domains`, respectively,
    /// and whose groupings abide by the descriptors in `margins`,
    /// or an error.
    pub(crate) fn new_with_margins(
        series_domains: Vec<SeriesDomain>,
        margins: HashMap<BTreeSet<PlSmallStr>, Margin>,
    ) -> Fallible<Self> {
        let n_unique = series_domains
            .iter()
            .map(|s| &s.name)
            .collect::<HashSet<_>>()
            .len();
        if n_unique != series_domains.len() {
            return fallible!(MakeDomain, "column names must be distinct");
        }
        Ok(FrameDomain {
            series_domains,
            margins,
            _marker: PhantomData,
        })
    }

    /// Create a new FrameDomain that follows a schema.
    ///
    /// # Proof Definition
    /// Either returns a FrameDomain spanning all dataframes with a schema `schema`, or an error.
    pub fn new_from_schema(schema: Schema) -> Fallible<Self> {
        let series_domains = (schema.iter_fields())
            .map(SeriesDomain::new_from_field)
            .collect::<Fallible<_>>()?;
        FrameDomain::new(series_domains)
    }

    /// # Proof Definition
    /// Return the schema shared by all members of the domain.
    pub fn schema(&self) -> Schema {
        Schema::from_iter(
            self.series_domains
                .iter()
                .map(|s| Field::new(s.name.clone(), s.dtype())),
        )
    }

    /// # Proof Definition
    /// Return a FrameDomain equivalent to `self`,
    /// but whose carrier type (the type of a frame) is `FO`.
    pub(crate) fn cast_carrier<FO: Frame>(&self) -> FrameDomain<FO> {
        FrameDomain {
            series_domains: self.series_domains.clone(),
            margins: self.margins.clone(),
            _marker: PhantomData,
        }
    }

    /// # Proof Definition
    /// Return a FrameDomain that only includes those elements of `self` that,
    /// when grouped by `grouping_columns`, observes those descriptors in `margin`,
    /// or an error.
    #[must_use]
    pub fn with_margin<I: AsRef<str>>(
        mut self,
        grouping_columns: &[I],
        margin: Margin,
    ) -> Fallible<Self> {
        let grouping_columns =
            BTreeSet::from_iter(grouping_columns.iter().map(|v| v.as_ref().into()));
        if self.margins.contains_key(&grouping_columns) {
            return fallible!(MakeDomain, "margin already exists");
        }
        self.margins.insert(grouping_columns, margin);
        Ok(self)
    }

    /// # Proof Definition
    /// Return margin descriptors about `grouping_columns`
    /// that can be inferred from `self`.
    ///
    /// Best effort is made to derive more restrictive descriptors,
    /// but optimal inference of descriptors is not guaranteed.
    pub fn get_margin(&self, grouping_columns: &BTreeSet<PlSmallStr>) -> Margin {
        let mut margin = self
            .margins
            .get(grouping_columns)
            .cloned()
            .unwrap_or_default();

        let subset_margins = self
            .margins
            .iter()
            .filter(|(id, _)| id.is_subset(grouping_columns))
            .collect::<Vec<(&BTreeSet<_>, &Margin)>>();

        // the max_partition_* descriptors can take the minimum known value from any margin on a subset of the grouping columns
        margin.max_partition_length = (subset_margins.iter())
            .filter_map(|(_, m)| m.max_partition_length)
            .min();

        margin.max_partition_contributions = (subset_margins.iter())
            .filter_map(|(_, m)| m.max_partition_contributions)
            .min();

        let all_mnps = (self.margins.iter())
            .filter_map(|(set, m)| Some((set, m.max_num_partitions?)))
            .collect();

        // in the worst case, the max partition length is the product of the max partition lengths of the cover
        margin.max_num_partitions = find_min_covering(grouping_columns.clone(), all_mnps)
            .map(|cover| cover.values().try_fold(1u32, |acc, v| acc.inf_mul(v).ok()))
            .flatten();

        let all_mips = (self.margins.iter())
            .filter_map(|(set, m)| Some((set, m.max_influenced_partitions?)))
            .collect();

        // in the worst case, the max partition contributions is the product of the max partition contributions of the cover
        margin.max_influenced_partitions = find_min_covering(grouping_columns.clone(), all_mips)
            .map(|cover| cover.values().try_fold(1u32, |acc, v| acc.inf_mul(v).ok()))
            .flatten();

        // if keys or lengths are known about any higher-way marginal,
        // then the same is known about lower-way marginals
        margin.public_info = (self.margins.iter())
            .filter(|(id, _)| grouping_columns.is_subset(id))
            .map(|(_, margin)| margin.public_info)
            .max()
            .flatten();

        // with no grouping, the key-set is trivial/public
        if grouping_columns.is_empty() {
            margin.public_info.get_or_insert(MarginPub::Keys);
            margin.max_num_partitions = Some(1);
            margin.max_influenced_partitions = Some(1);
        }

        margin
    }

    pub fn series_domain(&self, name: PlSmallStr) -> Fallible<SeriesDomain> {
        self.series_domains
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| err!(MakeTransformation, "unrecognized column: {}", name))
    }
}

impl<F: Frame> Debug for FrameDomain<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let margins_debug = self
            .margins
            .keys()
            .map(|id| format!("{:?}", id))
            .collect::<Vec<_>>()
            .join(", ");

        write!(
            f,
            "FrameDomain({}; margins=[{}])",
            self.series_domains
                .iter()
                .map(|s| format!("{}: {}", s.name, s.dtype()))
                .collect::<Vec<_>>()
                .join(", "),
            margins_debug
        )
    }
}

impl<F: Frame> Domain for FrameDomain<F> {
    type Carrier = F;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let val_df = val.clone().dataframe()?;

        // if df has different number of columns as domain
        if val_df.schema().len() != self.series_domains.len() {
            return Ok(false);
        }

        // check if each column is a member of the series domain
        for (series_domain, series) in self.series_domains.iter().zip(
            val_df
                .get_columns()
                .iter()
                .map(|c| c.as_materialized_series()),
        ) {
            if !series_domain.member(series)? {
                return Ok(false);
            }
        }

        // check that margins are satisfied
        for (grouping_columns, margin) in self.margins.iter() {
            let grouping_columns = grouping_columns
                .iter()
                .cloned()
                .map(col)
                .collect::<Vec<_>>();
            if !margin.member(val.clone().lazyframe().group_by(grouping_columns))? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A restriction on the unique values in the margin, as well as possibly their counts,
/// over a set of columns in a LazyFrame.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Margin {
    /// The greatest number of records that can be present in any one partition.
    pub max_partition_length: Option<u32>,
    /// The greatest number of partitions that can be present.
    pub max_num_partitions: Option<u32>,

    /// The greatest number of contributions that can be made by one unit to any one partition.
    ///
    /// This affects how margins interact with the metric.
    /// The distance between data sets differing by more than this quantity is considered infinite.
    pub max_partition_contributions: Option<u32>,
    /// The greatest number of partitions that can be contributed to.
    ///
    /// This affects how margins interact with the metric.
    /// The distance between data sets differing by more than this quantity is considered infinite.
    pub max_influenced_partitions: Option<u32>,

    /// Denote whether the unique values and/or in the margin are public.
    pub public_info: Option<MarginPub>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord)]
/// Denote how margins interact with the metric.
///
/// Order of elements in the enum is significant:
/// variants are ordered by how restrictive they are as descriptors.
pub enum MarginPub {
    /// The distance between data sets with different margin keys are is infinite.
    Keys,
    /// The distance between data sets with different margin keys or partition lengths is infinite.
    Lengths,
}

impl PartialOrd for MarginPub {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

impl Margin {
    pub fn with_max_partition_length(mut self, value: u32) -> Self {
        self.max_partition_length = Some(value);
        self
    }
    pub fn with_max_num_partitions(mut self, value: u32) -> Self {
        self.max_num_partitions = Some(value);
        self
    }
    pub fn with_max_partition_contributions(mut self, value: u32) -> Self {
        self.max_partition_contributions = Some(value);
        self
    }
    pub fn with_max_influenced_partitions(mut self, value: u32) -> Self {
        self.max_influenced_partitions = Some(value);
        self
    }

    pub fn with_public_keys(mut self) -> Self {
        self.public_info = Some(MarginPub::Keys);
        self
    }

    pub fn with_public_lengths(mut self) -> Self {
        self.public_info = Some(MarginPub::Lengths);
        self
    }

    /// # Proof Definition
    /// Only returns `Ok(true)` if the grouped data `value` is consistent with the domain descriptors in `self`.
    pub(crate) fn member(&self, value: LazyGroupBy) -> Fallible<bool> {
        // retrieves the first row/first column from $tgt as type $ty
        macro_rules! item {
            ($tgt:expr, $ty:ident) => {
                ($tgt.collect()?.get_columns()[0].$ty()?.get(0))
                    .ok_or_else(|| err!(FailedFunction))?
            };
        }

        let max_part_length = value.clone().agg([len()]).select(&[max("len")]);

        if item!(max_part_length, u32) > self.max_partition_length.unwrap_or(u32::MAX) {
            return Ok(false);
        }

        let max_num_parts = value.agg([]).select(&[len()]);

        if item!(max_num_parts, u32) > self.max_num_partitions.unwrap_or(u32::MAX) {
            return Ok(false);
        }
        Ok(true)
    }

    /// # Proof Definition
    /// Returns the greatest number of partitions that may differ
    /// when at most `l_1` records may be added or removed,
    /// given optional domain descriptor `max_num_partitions`
    /// and optional metric descriptor `max_influenced_partitions`.
    pub(crate) fn l_0(&self, l_1: u32) -> u32 {
        self.max_influenced_partitions
            .or(self.max_num_partitions)
            .unwrap_or(l_1)
            .min(l_1)
    }

    /// # Proof Definition
    /// Returns the greatest number of records that may be added or removed in any any one partition
    /// when at most `l_1` records may be added or removed,
    /// given optional domain descriptor `max_partition_length`
    /// and optional metric descriptor `max_partition_contributions`.
    pub(crate) fn l_inf(&self, l_1: u32) -> u32 {
        self.max_partition_contributions
            .or(self.max_partition_length)
            .unwrap_or(l_1)
            .min(l_1)
    }
}

/// # Proof Definition
/// Return a subset of `sets` whose intersection is a superset of `must_cover`.
///
/// While this algorithm also tries to minimize the number of sets returned,
/// finding the optimal smallest set of sets is not a requirement to prove correctness of this algorithm.
/// In fact, finding the optimal subset of sets is computationally infeasible, as the minimal set covering problem is NP-hard.
///
/// # Citation
/// * A Greedy Heuristic for the Set-Covering Problem, V. Chvatal
fn find_min_covering<T: Hash + Ord>(
    mut must_cover: BTreeSet<T>,
    sets: HashMap<&BTreeSet<T>, u32>,
) -> Option<HashMap<&BTreeSet<T>, u32>> {
    let mut covered = HashMap::<&BTreeSet<T>, u32>::new();
    while !must_cover.is_empty() {
        let (best_set, weight) = sets
            .iter()
            .max_by_key(|(set, len)| {
                (
                    // choose the set that covers the most uncovered elements
                    set.intersection(&must_cover).count(),
                    // of those, prioritize smaller sets
                    -(set.len() as i32),
                    // of those, prioritize lower weight
                    -(**len as i32),
                )
            })
            .and_then(|(&best_set, weight)| {
                (!best_set.is_disjoint(&must_cover)).then(|| (best_set, *weight))
            })?;

        must_cover.retain(|x| !best_set.contains(x));
        covered.insert(best_set, weight);
    }
    Some(covered)
}
