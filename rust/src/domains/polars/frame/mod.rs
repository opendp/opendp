use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use opendp_derive::proven;
use polars::lazy::dsl::len;
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::{FrameDistance, FrameDistanceFor, LInfDistance, LpDistance, MicrodataMetric};
use crate::traits::{InfMul, ProductOrd};
use crate::transformations::traits::UnboundedMetric;
use crate::{core::MetricSpace, domains::{ColumnDomain, SeriesDomain}, error::Fallible};

use super::NumericDataType;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

/// Frame carriers share column and grouping descriptors but may have
/// different execution and membership implementations.
pub trait Frame: Clone + Send + Sync + Sized {
    type Column: Clone + Send + Sync;
    type Expr: Clone + Eq + Hash + Debug;

    /// Only Polars-backed carriers need to override these conversions.
    fn lazyframe(self) -> Fallible<LazyFrame> {
        fallible!(FailedFunction, "no Polars LazyFrame conversion for this carrier")
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        fallible!(FailedFunction, "no Polars DataFrame conversion for this carrier")
    }

    /// Columns referenced by a grouping expression.
    fn root_names(expr: &Self::Expr, columns: &[ColumnDomain<Self::Column>]) -> Fallible<Vec<PlSmallStr>>;

    /// Carrier-specific membership, including column values and margins.
    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool>;
}

impl Frame for LazyFrame {
    type Column = Series;
    type Expr = Expr;
    fn lazyframe(self) -> Fallible<LazyFrame> { Ok(self) }
    fn dataframe(self) -> Fallible<DataFrame> { self.collect().map_err(Into::into) }
    fn root_names(expr: &Expr, _columns: &[SeriesDomain]) -> Fallible<Vec<PlSmallStr>> { Ok(expr.clone().meta().root_names()) }
    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool> {
        polars_frame_member(self, domain)
    }
}

impl Frame for DslPlan {
    type Column = Series;
    type Expr = Expr;
    fn lazyframe(self) -> Fallible<LazyFrame> { Ok(LazyFrame::from(self)) }
    fn dataframe(self) -> Fallible<DataFrame> { self.lazyframe()?.collect().map_err(Into::into) }
    fn root_names(expr: &Expr, _columns: &[SeriesDomain]) -> Fallible<Vec<PlSmallStr>> { Ok(expr.clone().meta().root_names()) }
    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool> {
        polars_frame_member(self, domain)
    }
}

impl Frame for DataFrame {
    type Column = Series;
    type Expr = Expr;
    fn lazyframe(self) -> Fallible<LazyFrame> { Ok(IntoLazy::lazy(self)) }
    fn dataframe(self) -> Fallible<DataFrame> { Ok(self) }
    fn root_names(expr: &Expr, _columns: &[SeriesDomain]) -> Fallible<Vec<PlSmallStr>> { Ok(expr.clone().meta().root_names()) }
    fn frame_member(&self, domain: &FrameDomain<Self>) -> Fallible<bool> {
        polars_frame_member(self, domain)
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
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, Frame, Margin, Invariant::*};
/// use std::collections::HashSet;
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
///         .with_margin(Margin::by(["A"]).with_invariant_keys())?
///         .with_margin(Margin::by(["B"]).with_invariant_lengths())?;
///
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone)]
pub struct FrameDomain<F: Frame> {
    pub series_domains: Vec<ColumnDomain<F::Column>>,
    pub margins: Vec<Margin<F::Expr>>,
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

// The VTable/SymmetricDistance instance of this compatibility contract is
// verified in frame/verus.rs. The generic Frame implementation remains trusted.
impl<F: Frame, M: MicrodataMetric> MetricSpace for (FrameDomain<F>, M) {
    fn check_space(&self) -> Fallible<()> {
        if let Some(identifier) = self.1.identifier() {
            identifier
                .meta()
                .root_names()
                .into_iter()
                .try_for_each(|n| self.0.series_domain(n).map(|_| ()))?;
        }
        Ok(())
    }
}

impl<F: Frame, M: UnboundedMetric> MetricSpace for (FrameDomain<F>, FrameDistance<M>) {
    fn check_space(&self) -> Fallible<()> {
        (self.0.clone(), self.1.0.clone()).check_space()
    }
}

impl<F: Frame, M: UnboundedMetric> MetricSpace
    for (FrameDomain<F>, FrameDistanceFor<M, F::Expr>)
{
    fn check_space(&self) -> Fallible<()> {
        (self.0.clone(), self.1.0.clone()).check_space()
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
    pub fn new(series_domains: Vec<ColumnDomain<F::Column>>) -> Fallible<Self> {
        Self::new_with_margins(series_domains, Vec::new())
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
        series_domains: Vec<ColumnDomain<F::Column>>,
        margins: Vec<Margin<F::Expr>>,
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

    /// # Proof Definition
    /// Return a FrameDomain equivalent to `self`,
    /// but whose carrier type (the type of a frame) is `FO`.
    pub(crate) fn cast_carrier<FO: Frame<Expr = F::Expr>>(&self) -> FrameDomain<FO> {
        FrameDomain {
            series_domains: self.series_domains.iter().map(ColumnDomain::cast_carrier).collect(),
            margins: self.margins.clone(),
            _marker: PhantomData,
        }
    }

    /// # Proof Definition
    /// Errors if another margin with same `by` keys is present,
    /// otherwise returns an equivalent FrameDomain, but with `margin`.
    #[must_use]
    pub fn with_margin(mut self, margin: Margin<F::Expr>) -> Fallible<Self> {
        for expr in &margin.by {
            for name in F::root_names(expr, &self.series_domains)? {
                self.series_domain(name)?;
            }
        }

        if self.margins.iter().any(|m| m.by == margin.by) {
            return fallible!(
                MakeDomain,
                "margin ({:?}) is already present in domain",
                margin.by
            );
        }
        self.margins.push(margin);
        Ok(self)
    }

    #[proven]
    /// # Proof Definition
    /// Return margin descriptors when grouped by `by`
    /// that can be inferred from `self`.
    ///
    /// Best effort is made to derive more restrictive descriptors,
    /// but optimal inference of descriptors is not guaranteed.
    pub fn get_margin(&self, by: &HashSet<F::Expr>) -> Margin<F::Expr> {
        // find the margin descriptor for `by` if it exists, otherwise create a new one
        let mut margin = (self.margins.iter())
            .find(|m| &m.by == by)
            .cloned()
            .unwrap_or_else(|| Margin::by(by.iter().cloned().collect::<Vec<_>>()));

        // find margins for coarser groupings of the data
        let coarser_margins = (self.margins.iter())
            .filter(|m| m.by.is_subset(by))
            .collect::<Vec<&Margin<F::Expr>>>();

        // the max_length is the largest group length of any coarser grouping
        margin.max_length = coarser_margins.iter().filter_map(|m| m.max_length).min();

        let all_max_groups = (self.margins.iter())
            .filter_map(|m| Some((&m.by, m.max_groups?)))
            .collect();

        // in the worst case, the max group length is the product of the max group lengths of the cover
        margin.max_groups = find_min_covering(by.clone(), all_max_groups).and_then(|cover| {
            cover
                .iter()
                .try_fold(1u32, |acc, (_, v)| acc.inf_mul(v).ok())
        });

        // if keys or lengths are known about any higher-way marginal,
        // then the same is known about lower-way marginals
        margin.invariant = (self.margins.iter())
            .filter(|m| by.is_subset(&m.by))
            .map(|m| m.invariant)
            .max()
            .flatten();

        margin
    }

    pub fn series_domain(&self, name: PlSmallStr) -> Fallible<ColumnDomain<F::Column>> {
        self.series_domains
            .iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or_else(|| {
                err!(
                    MakeTransformation,
                    "unrecognized column in series domain: {}",
                    name
                )
            })
    }
}

// These methods execute Polars plans or expose Polars schemas, and are not
// part of the common domain abstraction.
impl<F: Frame<Column = Series, Expr = Expr>> FrameDomain<F> {
    /// Create a new FrameDomain that follows a schema.
    ///
    /// # Proof Definition
    /// Either returns a FrameDomain spanning all dataframes with a schema `schema`, or an error.
    pub fn new_from_schema(schema: Schema) -> Fallible<Self> {
        let series_domains = (schema.iter_fields())
            .map(ColumnDomain::<F::Column>::new_from_field)
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
    /// Return the schema shared by all members of the domain,
    /// when `plan` is applied to members of the domain.
    pub(crate) fn simulate_schema(
        &self,
        plan: impl Fn(LazyFrame) -> LazyFrame,
    ) -> Fallible<Schema> {
        let output = plan(DataFrame::empty_with_schema(&self.schema()).lazy())
            .collect()
            .map_err(|e| {
                err!(
                    MakeTransformation,
                    "Failed to determine output dtypes: {}",
                    e
                )
            })?;
        Ok((&**output.schema()).clone())
    }
}

impl<F: Frame> Debug for FrameDomain<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let margins_debug = self
            .margins
            .iter()
            .map(|m| format!("{:?}", m.by))
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
    fn member(&self, val: &F) -> Fallible<bool> {
        val.frame_member(self)
    }
}

// Shared Polars implementation; verified plan carriers supply their own membership.
fn polars_frame_member<F: Frame<Column = Series, Expr = Expr>>(
    val: &F, domain: &FrameDomain<F>,
) -> Fallible<bool> {
    let val_df = val.clone().dataframe()?;
    if val_df.schema().len() != domain.series_domains.len() {
        return Ok(false);
    }
    for (column_domain, series) in domain.series_domains.iter().zip(
        val_df.get_columns().iter().map(|c| c.as_materialized_series()),
    ) {
        if !column_domain.member(series)? {
            return Ok(false);
        }
    }
    for margin in &domain.margins {
        if !margin.member(val.clone().lazyframe()?)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A restriction on the unique values in the margin, as well as possibly their counts,
/// over a set of columns in a LazyFrame.
#[derive(Clone, PartialEq, Debug)]
pub struct Margin<E: Eq + Hash = Expr> {
    /// The columns to group by to form the margin.
    pub by: HashSet<E>,

    /// The greatest number of records that can be present in any one group.
    pub max_length: Option<u32>,
    /// The greatest number of groups that can be present.
    pub max_groups: Option<u32>,

    /// Denote whether all datasets have the same keys and/or lengths.
    ///
    /// This is more general than a domain descriptor;
    /// it denotes a multiverse of potential domains.
    pub invariant: Option<Invariant>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord)]
/// Denote how margins interact with the metric.
///
/// Order of elements in the enum is significant:
/// variants are ordered by how restrictive they are as descriptors.
pub enum Invariant {
    /// All datasets share the same group keys.
    Keys,
    /// All datasets share the same group keys and group lengths.
    Lengths,
    // `Order` is also a potential invariant, for dropping the shuffle after collect.
}

impl PartialOrd for Invariant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

impl<E: Eq + Hash + Clone> Margin<E> {
    pub fn select() -> Self {
        Self::by::<&[E], E>(&[])
    }

    pub fn by<K: AsRef<[IE]>, IE: Into<E> + Clone>(by: K) -> Self {
        Self {
            by: by.as_ref().iter().cloned().map(Into::into).collect(),
            max_length: None,
            max_groups: None,
            invariant: None,
        }
    }

    pub fn with_max_length(mut self, value: u32) -> Self {
        self.max_length = Some(value);
        self
    }
    pub fn with_max_groups(mut self, value: u32) -> Self {
        self.max_groups = Some(value);
        self
    }

    pub fn with_invariant_keys(mut self) -> Self {
        self.invariant = Some(Invariant::Keys);
        self
    }

    pub fn with_invariant_lengths(mut self) -> Self {
        self.invariant = Some(Invariant::Lengths);
        self
    }

    /// # Proof Definition
    /// Returns the greatest number of groups that may differ
    /// when at most `l_1` records may be added or removed,
    /// given optional domain descriptor `max_groups`.
    pub(crate) fn l_0(&self, l_1: u32) -> u32 {
        self.max_groups.unwrap_or(l_1).min(l_1)
    }

    /// # Proof Definition
    /// Returns the greatest number of records that may be added or removed in any any one group
    /// when at most `l_1` records may be added or removed,
    /// given optional domain descriptor `max_length`.
    pub(crate) fn l_inf(&self, l_1: u32) -> u32 {
        self.max_length.unwrap_or(l_1).min(l_1)
    }
}

impl Margin<Expr> {
    /// # Proof Definition
    /// Only returns `Ok(true)` if the grouped data `value` is consistent with the domain descriptors in `self`.
    pub(crate) fn member(&self, value: LazyFrame) -> Fallible<bool> {
        // retrieves the first row/first column from $tgt as type $ty
        macro_rules! item {
            ($tgt:expr, $ty:ident) => {
                ($tgt.collect()?.get_columns()[0].$ty()?.get(0))
                    .ok_or_else(|| err!(FailedFunction))?
            };
        }

        // No grouping keys corresponds to select, with one group even on an empty frame.
        if self.by.is_empty() {
            if self.max_groups == Some(0) {
                return Ok(false);
            }
            let length = item!(value.select([len()]), u32);
            return Ok(length <= self.max_length.unwrap_or(u32::MAX));
        }

        let value = value.group_by(self.by.iter().cloned().collect::<Vec<_>>());
        let max_part_length = value
            .clone()
            .agg([len()])
            .select(&[max("len").fill_null(lit(0u32))]);

        if item!(max_part_length, u32) > self.max_length.unwrap_or(u32::MAX) {
            return Ok(false);
        }

        let max_num_parts = value.agg([]).select(&[len()]);

        if item!(max_num_parts, u32) > self.max_groups.unwrap_or(u32::MAX) {
            return Ok(false);
        }
        Ok(true)
    }


}

#[proven]
/// # Proof Definition
/// Return a subset of `sets` whose intersection contains `must_cover`, or None.
///
/// While this algorithm also tries to minimize the number of sets returned,
/// finding the optimal smallest set of sets is not a requirement to prove correctness of this algorithm.
/// In fact, finding the optimal subset of sets is computationally infeasible, as the minimal set covering problem is NP-hard.
///
/// # Citation
/// * A Greedy Heuristic for the Set-Covering Problem, V. Chvatal
pub(crate) fn find_min_covering<T: Hash + Eq>(
    mut must_cover: HashSet<T>,
    sets: Vec<(&HashSet<T>, u32)>,
) -> Option<Vec<(&HashSet<T>, u32)>> {
    let mut covered = Vec::<(&HashSet<T>, u32)>::new();
    while !must_cover.is_empty() {
        let (best_set, weight) = sets
            .iter()
            .max_by_key(|(set, weight)| {
                (
                    // choose the set that covers the most uncovered elements
                    set.intersection(&must_cover).count(),
                    // of those, prioritize smaller sets
                    -(set.len() as i32),
                    // of those, prioritize lower weight
                    -(*weight as i32),
                )
            })
            // If sets is non-empty, and the "best set" overlaps with the must_cover set,
            //    then it is a valid addition to the covering.
            // Otherwise, return None.
            .and_then(|(best_set, weight)| {
                (!best_set.is_disjoint(&must_cover)).then(|| (best_set, *weight))
            })?;

        must_cover.retain(|x| !best_set.contains(x));
        covered.push((&best_set, weight));
    }
    Some(covered)
}
