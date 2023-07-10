use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;

use polars::lazy::dsl::{col, cols, len};
use polars::prelude::*;

use crate::core::Domain;
use crate::metrics::AbsoluteDistance;
use crate::traits::TotalOrd;
use crate::transformations::item;
use crate::{
    core::MetricSpace,
    domains::{AtomDomain, OptionDomain, SeriesDomain},
    error::Fallible,
    transformations::DatasetMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

// gradations of public info:
//                | public keys | private keys
// public counts  | Some(df+id) | x
// partial counts | Some(df)    | x

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
/// `FrameDomain(F)` is the domain of all values of type `F`.
/// * `series_domains` - Vector of Series Domains .
/// * `margins` - Hash map of public information on data stored in the Frame.
/// `LazyFrameDomain` is a `FrameDomain(LazyFrame)` and represents all values of type `LazyFrame`.
/// `DataFrameDomain` is a `FrameDomain(DataFrame)` and represents all values of type `DataFrame`.
///
/// ## Generics
/// * `F` - Frame type: DataFrame or LazyFrame.
///
/// # Example
/// ```
/// use polars::prelude::*;
/// use opendp::domains::{AtomDomain, SeriesDomain, LazyFrameDomain, DataFrameDomain, Frame};
///
/// // Create a DataFrameDomain
/// let data_frame_domain = DataFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// // Create a LazyFrameDomain
/// let lazy_frame_domain = LazyFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<f64>::default()),
/// ])?;
///
/// // Create a LazyFrameDomain with Margins
/// let lazy_frame_domain_with_margins = LazyFrameDomain::new(vec![
///             SeriesDomain::new("A", AtomDomain::<i32>::default()),
///             SeriesDomain::new("B", AtomDomain::<String>::default()),
///         ])?
///         .with_counts(df!["A" => [1, 2], "count" => [1u32, 2]]?.lazyframe())?
///         .with_counts(df!["B" => ["1", "2"], "count" => [2u32, 1]]?.lazyframe())?;
///
/// # opendp::error::Fallible::Ok(())
/// ```
#[derive(Clone)]
pub struct FrameDomain<F: Frame> {
    pub series_domains: Vec<SeriesDomain>,
    pub margins: HashMap<BTreeSet<String>, Margin<F>>,
}

impl<F: Frame> PartialEq for FrameDomain<F> {
    fn eq(&self, other: &Self) -> bool {
        self.series_domains == other.series_domains && self.margins == other.margins
    }
}
pub type LazyFrameDomain = FrameDomain<LazyFrame>;
pub type DataFrameDomain = FrameDomain<DataFrame>;

impl<F: Frame, D: DatasetMetric> MetricSpace for (FrameDomain<F>, D) {
    fn check_space(&self) -> Fallible<()> {
        if D::SIZED {
            (self.0.margins.values())
                .find(|m| m.counts.is_some())
                .map(|_| ())
                .ok_or_else(|| err!(MetricSpace, "bounded dataset metric must have known size"))
        } else {
            Ok(())
        }
    }
}

impl<F: Frame, Q: TotalOrd> MetricSpace for (FrameDomain<F>, AbsoluteDistance<Q>) {
    fn check_space(&self) -> Fallible<()> {
        let margin = (self.0.margins)
            .get(&BTreeSet::new())
            .ok_or_else(|| err!(MetricSpace, "absolute distance must know dataframe margin"))?;

        let df = margin.data.clone().dataframe()?;

        if df.get_columns().len() != 1 {
            return fallible!(
                MetricSpace,
                "absolute distance must be over a single column"
            );
        }

        let vec = df.get_columns()[0].u32()?.to_vec();
        if vec.len() != 1 {
            return fallible!(MetricSpace, "absolute distance must be over a single group");
        }

        let count = vec[0].ok_or_else(|| err!(MetricSpace, "count must not be none"))?;

        if count != 1 {
            return fallible!(MetricSpace, "absolute distance must be over a single row");
        }

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
        })
    }

    pub fn schema(&self) -> Schema {
        Schema::from_iter(self.series_domains.iter().map(|sd| sd.field.clone()))
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

    // add categories to the domain
    #[must_use]
    pub fn with_categories(self, categories: Series) -> Fallible<Self> {
        let count_col_name = categories.name();
        // make sure the dtype matches
        self.check_dtype_matches(count_col_name, categories.dtype())?;

        let margin_id = BTreeSet::from_iter([categories.name().to_string()]);
        let margin = Margin::new_from_categories(categories)?;
        self.with_margin(margin_id, margin)
    }

    #[must_use]
    pub fn with_counts(self, counts: F) -> Fallible<Self> {
        let counts_schema = counts.schema()?;

        // determine which column is the counts column (the one not in the data)
        let counts_col_name = (counts_schema.iter_names())
            .find(|name| self.column(name).is_none())
            .ok_or_else(|| err!(MakeDomain, "could not find counts column"))?
            .to_string();

        let margin = Margin::new_from_counts(counts, counts_col_name)?;

        // check that all dtypes in id columns match
        let id_names = margin.get_join_column_names()?;
        for id_name in &id_names {
            self.check_dtype_matches(id_name, &counts_schema.try_get_field(id_name)?.dtype)?;
        }

        let margin_id = BTreeSet::from_iter(id_names);
        self.with_margin(margin_id, margin)
    }

    #[must_use]
    fn with_margin(mut self, margin_id: BTreeSet<String>, margin: Margin<F>) -> Fallible<Self> {
        if self.margins.contains_key(&margin_id) {
            return fallible!(MakeDomain, "margin already exists");
        }
        self.margins.insert(margin_id, margin);
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

    fn check_dtype_matches<I: AsRef<str>>(&self, name: I, dtype: &DataType) -> Fallible<()> {
        let domain_dtype = &self.try_column(&name)?.field.dtype;
        if domain_dtype != dtype {
            return fallible!(
                MakeDomain,
                "{} data type mismatch: expected {}, got {}",
                name.as_ref(),
                domain_dtype,
                dtype
            );
        }
        Ok(())
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
        for margin in self.margins.values() {
            if !margin.member(val)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// A restriction on the unique values in the margin, as well as possibly their counts,
/// over a set of columns in a LazyFrame.
///
/// If `counts_index` is not set, then `data` is the unique values in a column.
/// Otherwise, counts are stored in the `counts_index` column of the `data`.
#[derive(Clone)]
pub struct Margin<F: Frame> {
    pub data: F,
    pub counts: Option<String>,
}

impl<F: Frame> Margin<F> {
    pub fn new_from_categories(series: Series) -> Fallible<Self> {
        if series.n_unique()? != series.len() {
            return fallible!(MakeDomain, "categories must be unique");
        }
        Ok(Self {
            data: F::new(vec![series])?,
            counts: None,
        })
    }
    pub fn new_from_counts(data: F, counts_name: String) -> Fallible<Self> {
        let margin = Self {
            data,
            counts: Some(counts_name),
        };

        // set the data type on the counts column
        let count_col_name = margin.get_count_column_name()?;
        let count_col_dtype = margin.data.schema()?.try_get(&count_col_name)?.clone();
        if count_col_dtype != DataType::UInt32 {
            return fallible!(
                MakeDomain,
                "expected UInt32 counts, got {:?}",
                count_col_dtype
            );
        }

        Ok(margin)
    }

    fn get_count_column_name(&self) -> Fallible<String> {
        (self.counts.as_ref().cloned()).ok_or_else(|| err!(FailedFunction, "counts do not exist"))
    }

    fn get_join_column_names(&self) -> Fallible<Vec<String>> {
        Ok((self.data.schema()?.iter_names())
            .filter(|name| Some(name.as_str()) != self.counts.as_ref().map(|c| c.as_str()))
            .map(|v| v.to_string())
            .collect())
    }

    fn member(&self, value: &F) -> Fallible<bool> {
        let col_names = self.get_join_column_names()?;

        // 1. count number of unique combinations of col_names in actual data
        let actual_n_unique = item::<u32>(
            (value.clone().lazyframe())
                // .drop_nulls(Some(vec![cols(&col_names)])) // commented because counts for null values are permitted
                .select([as_struct(vec![cols(&col_names)]).n_unique()]),
        )?;
        // println!("actual n unique, {}", actual_n_unique);

        // 2. count number of unique combinations after an outer join with metadata
        let on_expr: Vec<_> = col_names.iter().map(|v| col(v.as_str())).collect();

        let actual_margins = (value.clone().lazyframe().group_by([cols(&col_names)])).agg([len()]);
        let joined = (self.data.clone().lazyframe()).join(
            actual_margins,
            on_expr.clone(),
            on_expr,
            JoinType::Left.into(),
        );

        // println!("joined {}", joined.clone().collect()?);

        // 3. to check that categories match, ensure that 1 and 2 are same length
        let joined_n_unique = item::<u32>(joined.clone().select([len()]))?;

        // if the join reduced the number of records,
        // then the actual data has values not in the category set
        if actual_n_unique != joined_n_unique {
            return Ok(false);
        }

        // 4. check that counts match
        if self.counts.is_some() {
            let count_colname = self.get_count_column_name()?;
            let count_colname_right = format!("{count_colname}_right");

            let eq_expr = col(count_colname.as_str())
                .eq(col(count_colname_right.as_str()))
                .all(true);

            if !item::<bool>(joined.clone().select([eq_expr]))? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl<F: Frame> PartialEq for Margin<F> {
    fn eq(&self, other: &Self) -> bool {
        if self.counts != other.counts {
            return false;
        }

        let Ok(self_margins) = self.data.clone().dataframe() else {
            return false;
        };
        let Ok(other_margins) = self.data.clone().dataframe() else {
            return false;
        };
        if self_margins != other_margins {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test_lazyframe {
    use crate::domains::{AtomDomain, OptionDomain};

    use super::*;

    #[test]
    fn test_frame_new() -> Fallible<()> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?;

        let frame = df!(
            "A" => &[3, 4, 5],
            "B" => &[1., 3., 7.]
        )?
        .lazyframe();

        assert!(frame_domain.member(&frame)?);

        Ok(())
    }

    #[test]
    fn test_frame_categories_bool() -> Fallible<()> {
        let categories = Series::new("A", vec![true]);
        let series_domain =
            SeriesDomain::new("A", OptionDomain::new(AtomDomain::<bool>::default()));
        let frame_domain =
            LazyFrameDomain::new(vec![series_domain])?.with_categories(categories.clone())?;

        // not a member because None is not in the category set
        let example = df!["A" => [Some(true), None]]?.lazyframe();
        assert!(!frame_domain.member(&example)?);

        let example = df!["A" => [Some(true), Some(false)]]?.lazyframe();
        assert!(!frame_domain.member(&example)?);

        let example = df!["A" => [Some(true)]]?.lazyframe();
        assert!(frame_domain.member(&example)?);

        Ok(())
    }

    #[test]
    fn test_frame_categories_float() -> Fallible<()> {
        let categories = Series::new("A", vec![1.]);
        let series_domain = SeriesDomain::new("A", OptionDomain::new(AtomDomain::<f64>::default()));
        let frame_domain =
            LazyFrameDomain::new(vec![series_domain])?.with_categories(categories.clone())?;

        let example = df!["A" => [Some(1.), None]]?.lazyframe();
        assert!(!frame_domain.member(&example)?);
        let example = df!["A" => [1., 2.]]?.lazyframe();
        assert!(!frame_domain.member(&example)?);
        let example = df!["A" => [1.]]?.lazyframe();
        assert!(frame_domain.member(&example)?);

        Ok(())
    }

    #[test]
    fn test_frame_counts() -> Fallible<()> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<String>::default()),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1u32, 2]]?.lazyframe())?
        .with_counts(df!["B" => ["1", "2"], "count" => [2u32, 1]]?.lazyframe())?;

        let frame = df!("A" => [1, 2, 2], "B" => ["1", "1", "2"])?.lazyframe();
        assert!(frame_domain.member(&frame)?);

        Ok(())
    }
}
