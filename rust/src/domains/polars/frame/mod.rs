use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;

use polars::lazy::dsl::{col, len};
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::{LInfDistance, LpDistance};
use crate::traits::ProductOrd;
use crate::{
    core::MetricSpace, domains::SeriesDomain, error::Fallible, transformations::DatasetMetric,
};

use super::NumericDataType;

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
mod ffi;

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
    pub margins: HashMap<BTreeSet<String>, Margin>,
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
            .any(|s| s.field.dtype != T::dtype())
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
        margins: HashMap<BTreeSet<String>, Margin>,
    ) -> Fallible<Self> {
        let n_unique = series_domains
            .iter()
            .map(|s| &s.field.name)
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
        Schema::from_iter(self.series_domains.iter().map(|s| s.field.clone()))
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
            BTreeSet::from_iter(grouping_columns.iter().map(|v| v.as_ref().to_string()));
        if self.margins.contains_key(&grouping_columns) {
            return fallible!(MakeDomain, "margin already exists");
        }
        self.margins.insert(grouping_columns, margin);
        Ok(self)
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
                .map(|s| format!("{}: {}", s.field.name, s.field.dtype))
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
        for (series_domain, series) in self.series_domains.iter().zip(val_df.get_columns().iter()) {
            if !series_domain.member(series)? {
                return Ok(false);
            }
        }

        // check that margins are satisfied
        for (grouping_columns, margin) in self.margins.iter() {
            let grouping_columns = grouping_columns.iter().map(|c| col(c)).collect::<Vec<_>>();
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

#[derive(Clone, PartialEq, Debug)]
/// Denote how margins interact with the metric.
pub enum MarginPub {
    /// The distance between data sets with different margin keys are is infinite.
    Keys,
    /// The distance between data sets with different margin keys or partition lengths is infinite.
    Lengths,
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
    fn member(&self, value: LazyGroupBy) -> Fallible<bool> {
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
