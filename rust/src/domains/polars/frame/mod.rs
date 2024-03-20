use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::marker::PhantomData;

use polars::lazy::dsl::{col, len};
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::LInfDistance;
use crate::{
    core::MetricSpace,
    domains::{AtomDomain, OptionDomain, SeriesDomain},
    error::Fallible,
    transformations::DatasetMetric,
};

pub trait Frame: Clone + Send + Sync {
    fn new(series: Vec<Series>) -> Fallible<Self>;
    fn schema(&self) -> Fallible<Schema>;
    fn lazyframe(self) -> LazyFrame;
    fn dataframe(self) -> Fallible<DataFrame>;
}
impl Frame for LazyFrame {
    fn new(series: Vec<Series>) -> Fallible<Self> {
        Ok(IntoLazy::lazy(DataFrame::new(series)?))
    }
    fn schema(&self) -> Fallible<Schema> {
        self.schema()
            .map(|v| v.as_ref().clone())
            .map_err(Into::into)
    }
    fn lazyframe(self) -> LazyFrame {
        self
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        self.collect().map_err(Into::into)
    }
}
impl Frame for LogicalPlan {
    fn new(series: Vec<Series>) -> Fallible<Self> {
        <LazyFrame as Frame>::new(series).map(|v| v.logical_plan)
    }
    fn schema(&self) -> Fallible<Schema> {
        Ok(self.schema()?.as_ref().as_ref().clone())
    }
    fn lazyframe(self) -> LazyFrame {
        LazyFrame::from(self)
    }
    fn dataframe(self) -> Fallible<DataFrame> {
        self.lazyframe().collect().map_err(Into::into)
    }
}
impl Frame for DataFrame {
    fn new(series: Vec<Series>) -> Fallible<Self> {
        Self::new(series).map_err(Into::into)
    }
    fn schema(&self) -> Fallible<Schema> {
        Ok(self.schema())
    }
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
///         .with_margin(&["A"], Margin::new().with_public_keys())?
///         .with_margin(&["B"], Margin::new().with_public_sizes())?;
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
pub(crate) type LogicalPlanDomain = FrameDomain<LogicalPlan>;

impl<F: Frame, M: DatasetMetric> MetricSpace for (FrameDomain<F>, M) {
    fn check_space(&self) -> Fallible<()> {
        if M::SIZED
            && !self
                .0
                .margins
                .values()
                .any(|m| m.public_info == Some(MarginPub::Lengths))
        {
            return fallible!(MetricSpace, "bounded dataset metric must have known size");
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
    pub fn new(series_domains: Vec<SeriesDomain>) -> Fallible<Self> {
        let num_unique = BTreeSet::from_iter(series_domains.iter().map(|s| &s.field.name)).len();
        if num_unique != series_domains.len() {
            return fallible!(MakeDomain, "column names must be distinct");
        }
        Ok(FrameDomain {
            series_domains,
            margins: HashMap::new(),
            _marker: PhantomData,
        })
    }

    pub(crate) fn cast_carrier<FO: Frame>(&self) -> FrameDomain<FO> {
        FrameDomain {
            series_domains: self.series_domains.clone(),
            margins: self.margins.clone(),
            _marker: PhantomData,
        }
    }

    pub fn new_from_schema(schema: Schema) -> Fallible<Self> {
        let series_domains = (schema.iter_fields())
            .map(|field| {
                macro_rules! new_series_domain {
                    ($ty:ty, $func:ident) => {
                        SeriesDomain::new(
                            field.name.as_str(),
                            OptionDomain::new(AtomDomain::<$ty>::$func()),
                        )
                    };
                }

                Ok(match field.data_type() {
                    DataType::Boolean => new_series_domain!(bool, default),
                    DataType::UInt8 => new_series_domain!(u8, default),
                    DataType::UInt16 => new_series_domain!(u16, default),
                    DataType::UInt32 => new_series_domain!(u32, default),
                    DataType::UInt64 => new_series_domain!(u64, default),
                    DataType::Int8 => new_series_domain!(i8, default),
                    DataType::Int16 => new_series_domain!(i16, default),
                    DataType::Int32 => new_series_domain!(i32, default),
                    DataType::Int64 => new_series_domain!(i64, default),
                    DataType::Float32 => new_series_domain!(f64, new_nullable),
                    DataType::Float64 => new_series_domain!(f64, new_nullable),
                    DataType::String => new_series_domain!(String, default),
                    dtype => return fallible!(MakeDomain, "unsupported type {}", dtype),
                })
            })
            .collect::<Fallible<_>>()?;
        FrameDomain::new(series_domains)
    }

    #[must_use]
    pub fn with_margin<I: AsRef<str>>(mut self, by: &[I], margin: Margin) -> Fallible<Self> {
        let by_set = BTreeSet::from_iter(by.iter().map(AsRef::as_ref).map(ToString::to_string));
        if by.len() != by_set.len() {
            return fallible!(MakeDomain, "margin columns must be distinct");
        }
        if self.margins.contains_key(&by_set) {
            return fallible!(MakeDomain, "margin already exists");
        }
        self.margins.insert(by_set, margin);
        Ok(self)
    }

    fn column_index<I: AsRef<str>>(&self, name: I) -> Option<usize> {
        self.series_domains
            .iter()
            .position(|s| s.field.name() == name.as_ref())
    }
    pub fn column<I: AsRef<str>>(&self, name: I) -> Option<&SeriesDomain> {
        self.column_index(name).map(|i| &self.series_domains[i])
    }
    pub fn try_column<I: AsRef<str>>(&self, name: I) -> Fallible<&SeriesDomain> {
        self.column(&name)
            .ok_or_else(|| err!(FailedFunction, "{} is not in dataframe", name.as_ref()))
    }
    pub fn try_column_mut<I: AsRef<str>>(&mut self, name: I) -> Fallible<&mut SeriesDomain> {
        let series_index = self
            .column_index(name.as_ref())
            .ok_or_else(|| err!(FailedFunction, "{} is not in dataframe", name.as_ref()))?;
        Ok(&mut self.series_domains[series_index])
    }

    pub fn schema(&self) -> Schema {
        Schema::from_iter(self.series_domains.iter().map(|s| s.field.clone()))
    }
}

impl<F: Frame> Debug for FrameDomain<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut margins_debug = self
            .margins
            .keys()
            .map(|id| format!("{:?}", id))
            .collect::<Vec<_>>()
            .join(", ");
        if !margins_debug.is_empty() {
            margins_debug = format!("; margins=[{}]", margins_debug);
        }
        write!(
            f,
            "LazyFrameDomain({}{})",
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
        for (s, dom) in val_df.get_columns().iter().zip(self.series_domains.iter()) {
            if !dom.member(s)? {
                return Ok(false);
            }
        }

        // check that margins are satisfied
        for (by, margin) in self.margins.iter() {
            if !margin.member(by, val.clone().lazyframe())? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A restriction on the unique values in the margin, as well as possibly their counts,
/// over a set of columns in a LazyFrame.
#[derive(Clone, PartialEq)]
pub struct Margin {
    /// The greatest number of records that can be present in any one partition.
    pub max_partition_size: Option<u32>,
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

#[derive(Clone, PartialEq)]
/// Denote how margins interact with the metric.
pub enum MarginPub {
    /// The distance between data sets with different margin keys are is infinite.
    Keys,
    /// The distance between data sets with different margin keys or partition lengths is infinite.
    Lengths,
}

impl Margin {
    pub fn new() -> Self {
        Margin {
            max_partition_size: None,
            max_num_partitions: None,
            max_partition_contributions: None,
            max_influenced_partitions: None,
            public_info: None,
        }
    }

    pub fn max_partition_size(mut self, value: u32) -> Self {
        self.max_partition_size = Some(value);
        self
    }
    pub fn max_num_partitions(mut self, value: u32) -> Self {
        self.max_num_partitions = Some(value);
        self
    }

    pub fn max_partition_contributions(mut self, value: u32) -> Self {
        self.max_partition_contributions = Some(value);
        self
    }
    pub fn max_influenced_partitions(mut self, value: u32) -> Self {
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

    fn member(&self, by: &BTreeSet<String>, val: LazyFrame) -> Fallible<bool> {
        // retrieves the first row/first column from $tgt as type $ty
        macro_rules! item {
            ($tgt:expr, $ty:ident) => {
                ($tgt.collect()?.get_columns()[0].$ty()?.get(0))
                    .ok_or_else(|| err!(FailedFunction))?
            };
        }

        let max_part_size = val
            .clone()
            .group_by(by.iter().map(|s| col(s)).collect::<Vec<_>>())
            .agg([len()])
            .select(&[max("len")]);

        if item!(max_part_size, u32) > self.max_partition_size.unwrap_or(u32::MAX) {
            return Ok(false);
        }

        let max_num_parts =
            val.select([cols(by.iter().map(|s| s.as_str()).collect::<Vec<_>>()).n_unique()]);

        if item!(max_num_parts, u32) > self.max_num_partitions.unwrap_or(u32::MAX) {
            return Ok(false);
        }
        Ok(true)
    }
}

#[cfg(test)]
mod test_lazyframe {
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
            Margin::new().max_partition_size(1).max_num_partitions(2),
        )?;

        let lf_exceed_partition_size = df!("A" => [1, 2, 2], "B" => ["1", "1", "2"])?.lazyframe();
        assert!(!lf_domain.member(&lf_exceed_partition_size)?);

        let lf_exceed_num_partitions = df!("A" => [1, 2, 3], "B" => ["1", "1", "1"])?.lazyframe();
        assert!(!lf_domain.member(&lf_exceed_num_partitions)?);

        let lf = df!("A" => [1, 2], "B" => ["1", "1"])?.lazyframe();
        assert!(lf_domain.member(&lf)?);

        Ok(())
    }
}
