use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;

use polars::lazy::dsl::{col, cols, count};
use polars::prelude::*;

use crate::core::Domain;
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

#[derive(Clone, PartialEq)]
pub struct LazyFrameDomain {
    pub series_domains: Vec<SeriesDomain>,
    pub margins: HashMap<BTreeSet<String>, Margin>,
}

impl<D: DatasetMetric> MetricSpace for (LazyFrameDomain, D) {
    fn check(&self) -> bool {
        true
    }
}

impl LazyFrameDomain {
    pub fn new(series_domains: Vec<SeriesDomain>) -> Fallible<Self> {
        let num_unique = BTreeSet::from_iter(series_domains.iter().map(|s| &s.field.name)).len();
        if num_unique != series_domains.len() {
            return fallible!(MakeDomain, "column names must be distinct");
        }
        Ok(LazyFrameDomain {
            series_domains,
            margins: HashMap::new(),
        })
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
                    DataType::Utf8 => new_series_domain!(String, default),
                    dtype => return fallible!(MakeDomain, "unsupported type {}", dtype),
                })
            })
            .collect::<Fallible<_>>()?;
        LazyFrameDomain::new(series_domains)
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
    pub fn with_counts(self, counts: LazyFrame) -> Fallible<Self> {
        let counts_schema = counts.schema()?;

        // determine which column is the counts column (the one not in the data)
        let counts_col_index = (counts_schema.iter_names())
            .position(|name| self.column(name).is_none())
            .ok_or_else(|| err!(MakeDomain, "could not find counts column"))?;

        let margin = Margin::new_from_counts(counts, counts_col_index)?;

        // check that all dtypes in id columns match
        let id_names = margin.get_join_column_names()?;
        for id_name in &id_names {
            self.check_dtype_matches(id_name, &counts_schema.try_get_field(id_name)?.dtype)?;
        }

        let margin_id = BTreeSet::from_iter(id_names);
        self.with_margin(margin_id, margin)
    }

    #[must_use]
    fn with_margin(mut self, margin_id: BTreeSet<String>, margin: Margin) -> Fallible<Self> {
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
        let series_index = self.column_index(name.as_ref())
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

impl Debug for LazyFrameDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LazyFrameDomain({})",
            self.series_domains
                .iter()
                .map(|s| format!("{}: {}", s.field.name, s.field.dtype))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Domain for LazyFrameDomain {
    type Carrier = LazyFrame;
    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        let val_df = val.clone().collect()?;

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
pub struct Margin {
    data: LazyFrame,
    counts_index: Option<usize>,
}

impl Margin {
    pub fn new_from_categories(series: Series) -> Fallible<Self> {
        if series.n_unique()? != series.len() {
            return fallible!(MakeDomain, "categories must be unique");
        }
        Ok(Self {
            data: DataFrame::new(vec![series])?.lazy(),
            counts_index: None,
        })
    }
    pub fn new_from_counts(data: LazyFrame, counts_index: usize) -> Fallible<Self> {
        let mut margin = Self {
            data,
            counts_index: Some(counts_index),
        };

        // set the data type on the counts column
        let count_col_name = margin.get_count_column_name()?;
        margin.data = margin
            .data
            .with_column(col(count_col_name.as_str()).cast(DataType::UInt32));

        Ok(margin)
    }

    fn get_count_column_name(&self) -> Fallible<String> {
        let count_index = self
            .counts_index
            .ok_or_else(|| err!(FailedFunction, "counts do not exist"))?;
        Ok((self.data.schema()?.get_at_index(count_index).unwrap().0).to_string())
    }

    fn get_join_column_names(&self) -> Fallible<Vec<String>> {
        Ok((self.data.schema()?.iter_names().enumerate())
            .filter(|(i, _)| Some(*i) != self.counts_index)
            .map(|v| v.1.to_string())
            .collect())
    }

    fn member(&self, value: &LazyFrame) -> Fallible<bool> {
        let col_names = self.get_join_column_names()?;

        // retrieves the first row/first column from $tgt as type $ty
        macro_rules! item {
            ($tgt:expr, $ty:ident) => {
                ($tgt.collect()?.get_columns()[0].$ty()?.get(0))
                    .ok_or_else(|| err!(FailedFunction))?
            };
        }

        // 1. count number of unique combinations of col_names in actual data
        let actual_n_unique = item!(
            (value.clone())
                // .drop_nulls(Some(vec![cols(&col_names)])) // commented because counts for null values are permitted
                .select([as_struct(&[cols(&col_names)]).n_unique()]),
            u32
        );
        // println!("actual n unique, {}", actual_n_unique);

        // 2. count number of unique combinations after an outer join with metadata
        let on_expr: Vec<_> = col_names.iter().map(|v| col(v.as_str())).collect();

        let actual_margins = (value.clone().groupby([cols(&col_names)])).agg([count()]);
        let joined =
            (self.data.clone()).join(actual_margins, on_expr.clone(), on_expr, JoinType::Left);

        // println!("joined {}", joined.clone().collect()?);

        // 3. to check that categories match, ensure that 1 and 2 are same length
        let joined_n_unique = item!(joined.clone().select([count()]), u32);

        // if the join reduced the number of records,
        // then the actual data has values not in the category set
        if actual_n_unique != joined_n_unique {
            return Ok(false);
        }

        // 4. check that counts match
        if self.counts_index.is_some() {
            let count_colname = self.get_count_column_name()?;
            let count_colname_right = format!("{count_colname}_right");

            let eq_expr = col(count_colname.as_str())
                .eq(col(count_colname_right.as_str()))
                .all();

            if !item!(joined.clone().select([eq_expr]), bool) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl PartialEq for Margin {
    fn eq(&self, other: &Self) -> bool {
        if self.counts_index != other.counts_index {
            return false;
        }

        let Ok(self_margins) = self.data.clone().collect() else {return false};
        let Ok(other_margins) = self.data.clone().collect() else {return false};
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
        .lazy();

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
        let example = df!["A" => [Some(true), None]]?.lazy();
        assert!(!frame_domain.member(&example)?);

        let example = df!["A" => [Some(true), Some(false)]]?.lazy();
        assert!(!frame_domain.member(&example)?);

        let example = df!["A" => [Some(true)]]?.lazy();
        assert!(frame_domain.member(&example)?);

        Ok(())
    }

    #[test]
    fn test_frame_categories_float() -> Fallible<()> {
        let categories = Series::new("A", vec![1.]);
        let series_domain = SeriesDomain::new("A", OptionDomain::new(AtomDomain::<f64>::default()));
        let frame_domain =
            LazyFrameDomain::new(vec![series_domain])?.with_categories(categories.clone())?;

        let example = df!["A" => [Some(1.), None]]?.lazy();
        assert!(!frame_domain.member(&example)?);
        let example = df!["A" => [1., 2.]]?.lazy();
        assert!(!frame_domain.member(&example)?);
        let example = df!["A" => [1.]]?.lazy();
        assert!(frame_domain.member(&example)?);

        Ok(())
    }

    #[test]
    fn test_frame_counts() -> Fallible<()> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<String>::default()),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => ["1", "2"], "count" => [2, 1]]?.lazy())?;

        let frame = df!("A" => [1, 2, 2], "B" => ["1", "1", "2"])?.lazy();
        assert!(frame_domain.member(&frame)?);

        Ok(())
    }
}
