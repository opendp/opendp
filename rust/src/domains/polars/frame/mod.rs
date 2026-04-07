use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use opendp_derive::proven;
use polars::lazy::dsl::len;
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::{
    DatabaseIdDistance, FrameDistance, LInfDistance, LpDistance, MicrodataMetric,
};
use crate::traits::{InfMul, ProductOrd};
use crate::transformations::traits::UnboundedMetric;
use crate::{core::MetricSpace, domains::SeriesDomain, error::Fallible};

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
    pub series_domains: Vec<SeriesDomain>,
    pub margins: Vec<Margin>,
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

pub const OPENDP_TABLE_NAME_PREFIX: &str = "__OPENDP_TABLE_NAME__";

#[derive(Clone, PartialEq)]
pub struct DatabaseDomain(pub HashMap<String, LazyFrameDomain>);

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

impl MetricSpace for (DatabaseDomain, DatabaseIdDistance) {
    fn check_space(&self) -> Fallible<()> {
        let mut has_protected_site = false;
        for (table_name, id_sites) in &self.1.bindings {
            let Some(frame_domain) = self.0.0.get(table_name) else {
                return fallible!(
                    MetricSpace,
                    "metric references unrecognized table: {}",
                    table_name
                );
            };
            has_protected_site |= id_sites.iter().any(|site| site.space == self.1.protect);
            for expr in id_sites.iter().flat_map(|site| site.exprs.iter()) {
                for root in expr.clone().meta().root_names() {
                    frame_domain.series_domain(root)?;
                }
            }
        }
        if !has_protected_site {
            return fallible!(
                MetricSpace,
                "database metric does not reference the protected identifier label: {}",
                self.1.protect
            );
        }
        Ok(())
    }
}

impl<F: Frame, M: UnboundedMetric> MetricSpace for (FrameDomain<F>, FrameDistance<M>) {
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
    pub fn new(series_domains: Vec<SeriesDomain>) -> Fallible<Self> {
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
        series_domains: Vec<SeriesDomain>,
        margins: Vec<Margin>,
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
    /// Errors if another margin with same `by` keys is present,
    /// otherwise returns an equivalent FrameDomain, but with `margin`.
    #[must_use]
    pub fn with_margin(mut self, margin: Margin) -> Fallible<Self> {
        (margin.by.iter())
            .map(|e| e.clone().meta().root_names())
            .flatten()
            .try_for_each(|name| self.series_domain(name).map(|_| ()))?;

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
    pub fn get_margin(&self, by: &HashSet<Expr>) -> Margin {
        // find the margin descriptor for `by` if it exists, otherwise create a new one
        let mut margin = (self.margins.iter())
            .find(|m| &m.by == by)
            .cloned()
            .unwrap_or_else(|| Margin::by(by.iter().cloned().collect::<Vec<_>>()));

        // find margins for coarser groupings of the data
        let coarser_margins = (self.margins.iter())
            .filter(|m| m.by.is_subset(by))
            .collect::<Vec<&Margin>>();

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

    pub fn series_domain(&self, name: PlSmallStr) -> Fallible<SeriesDomain> {
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

impl DatabaseDomain {
    pub fn new(table_domains: HashMap<String, LazyFrameDomain>) -> Self {
        Self(table_domains)
    }

    pub fn table(&self, table_name: &str) -> Fallible<LazyFrameDomain> {
        self.0.get(table_name).cloned().ok_or_else(|| {
            err!(
                MakeTransformation,
                "unrecognized table domain: {}",
                table_name
            )
        })
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

impl Debug for DatabaseDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DatabaseDomain({})",
            self.0
                .iter()
                .map(|(name, domain)| format!("{name}: {domain:?}"))
                .collect::<Vec<_>>()
                .join(", ")
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
        for margin in self.margins.iter() {
            let by = margin.by.iter().cloned().collect::<Vec<_>>();
            if !margin.member(val.clone().lazyframe().group_by(by))? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

fn table_marker_columns(schema: &Schema) -> Vec<PlSmallStr> {
    schema
        .iter_names()
        .filter(|name| name.as_str().starts_with(OPENDP_TABLE_NAME_PREFIX))
        .cloned()
        .collect()
}

pub(crate) fn strip_table_markers_from_schema(schema: &Schema) -> Schema {
    let mut stripped = schema.clone();
    for marker in table_marker_columns(schema) {
        stripped.remove(marker.as_str());
    }
    stripped
}

pub(crate) fn table_name_from_schema(schema: &Schema) -> Fallible<String> {
    let markers = table_marker_columns(schema);
    let [marker] = markers.as_slice() else {
        return fallible!(
            MakeTransformation,
            "expected exactly one source table marker column prefixed by {}, found {:?}",
            OPENDP_TABLE_NAME_PREFIX,
            markers
        );
    };

    let table_name = marker
        .as_str()
        .strip_prefix(OPENDP_TABLE_NAME_PREFIX)
        .ok_or_else(|| {
            err!(
                MakeTransformation,
                "table marker column must have form `{}table_name`, found `{}`",
                OPENDP_TABLE_NAME_PREFIX,
                marker
            )
        })?;

    Ok(table_name
        .strip_prefix('[')
        .and_then(|name| name.strip_suffix(']'))
        .unwrap_or(table_name)
        .to_string())
}

pub(crate) fn table_name_from_schema_or_domain(
    schema: &Schema,
    database_domain: &DatabaseDomain,
) -> Fallible<String> {
    table_name_from_schema(schema).or_else(|_| match database_domain.0.len() {
        1 => database_domain.0.keys().next().cloned().ok_or_else(|| {
            err!(
                MakeTransformation,
                "database domain unexpectedly had no tables"
            )
        }),
        _ => fallible!(
            MakeTransformation,
            "expected exactly one source table marker column for a multi-table query"
        ),
    })
}

pub(crate) fn strip_table_markers(df: &DataFrame) -> Fallible<DataFrame> {
    table_marker_columns(df.schema())
        .into_iter()
        .try_fold(df.clone(), |acc, marker| {
            acc.drop(marker.as_str()).map_err(Into::into)
        })
}

pub type Database = HashMap<String, LazyFrame>;
impl Domain for DatabaseDomain {
    type Carrier = Database;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        if self.0.len() != val.len() {
            return fallible!(FailedFunction);
        }

        for (name, frame) in val {
            let frame_domain = self.0.get(name).ok_or_else(|| err!(FailedFunction))?;
            let stripped = strip_table_markers(&frame.clone().collect()?)?;
            if !frame_domain.cast_carrier::<DataFrame>().member(&stripped)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A restriction on the unique values in the margin, as well as possibly their counts,
/// over a set of columns in a LazyFrame.
#[derive(Clone, PartialEq, Debug)]
pub struct Margin {
    /// The columns to group by to form the margin.
    pub by: HashSet<Expr>,

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

impl Margin {
    pub fn select() -> Margin {
        Self::by::<&[Expr], Expr>(&[])
    }

    pub fn by<E: AsRef<[IE]>, IE: Into<Expr> + Clone>(by: E) -> Self {
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
    /// Only returns `Ok(true)` if the grouped data `value` is consistent with the domain descriptors in `self`.
    pub(crate) fn member(&self, value: LazyGroupBy) -> Fallible<bool> {
        let max_part_length = value.clone().agg([len()]).select(&[max("len")]).collect()?;
        if max_part_length.height() == 0 {
            return Ok(true);
        }

        let max_part_length = max_part_length.get_columns()[0].u32()?.get(0).unwrap_or(0);

        if max_part_length > self.max_length.unwrap_or(u32::MAX) {
            return Ok(false);
        }

        let max_num_parts = value.agg([]).select(&[len()]).collect()?;
        if max_num_parts.height() == 0 {
            return Ok(true);
        }

        let max_num_parts = max_num_parts.get_columns()[0].u32()?.get(0).unwrap_or(0);

        if max_num_parts > self.max_groups.unwrap_or(u32::MAX) {
            return Ok(false);
        }
        Ok(true)
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
