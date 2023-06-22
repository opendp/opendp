use std::path::PathBuf;

use polars::prelude::*;

use crate::{
    core::{Domain, Metric, MetricSpace},
    error::Fallible,
    metrics::{
        ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

use super::LazyFrameDomain;

pub trait DatasetMetric: Metric<Distance = IntDistance> {
    const BOUNDED: bool;
}
impl DatasetMetric for SymmetricDistance {
    const BOUNDED: bool = false;
}
impl DatasetMetric for InsertDeleteDistance {
    const BOUNDED: bool = false;
}
impl DatasetMetric for ChangeOneDistance {
    const BOUNDED: bool = true;
}
impl DatasetMetric for HammingDistance {
    const BOUNDED: bool = true;
}

#[derive(Clone, PartialEq, Debug)]
pub struct CsvDomain {
    pub lazyframe_domain: LazyFrameDomain,
    pub delimiter: char,
    pub has_header: bool,
    pub skip_rows: usize,
    pub comment_char: Option<char>,
    pub quote_char: Option<char>,
    pub eol_char: char,
    pub null_values: Option<NullValues>,
}

impl CsvDomain {
    pub fn new(lazyframe_domain: LazyFrameDomain) -> Self {
        CsvDomain {
            lazyframe_domain,
            delimiter: ',',
            has_header: true,
            skip_rows: 0,
            comment_char: None,
            quote_char: Some('"'),
            eol_char: '\n',
            null_values: None,
        }
    }

    /// Set the CSV file's column delimiter as a byte character
    #[must_use]
    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Set whether the CSV file has headers
    #[must_use]
    pub fn has_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Skip the first `n` rows during parsing. The header will be parsed at row `n`.
    #[must_use]
    pub fn with_skip_rows(mut self, skip_rows: usize) -> Self {
        self.skip_rows = skip_rows;
        self
    }

    /// Set the comment character. Lines starting with this character will be ignored.
    #[must_use]
    pub fn with_comment_char(mut self, comment_char: Option<char>) -> Self {
        self.comment_char = comment_char;
        self
    }

    /// Set the `char` used as quote char. The default is `'"'`. If set to `[None]` quoting is disabled.
    #[must_use]
    pub fn with_quote_char(mut self, quote: Option<char>) -> Self {
        self.quote_char = quote;
        self
    }

    /// Set the `char` used as end of line. The default is `'\n'`.
    #[must_use]
    pub fn with_end_of_line_char(mut self, eol_char: char) -> Self {
        self.eol_char = eol_char;
        self
    }

    /// Set values that will be interpreted as missing/ null.
    #[must_use]
    pub fn with_null_values(mut self, null_values: Option<NullValues>) -> Self {
        self.null_values = null_values;
        self
    }

    pub fn new_reader<'a>(&self, path: PathBuf) -> LazyCsvReader<'a> {
        LazyCsvReader::new(path)
            // parsing errors are a side-channel
            .with_ignore_errors(true)
            // fill missing columns with null
            .with_missing_is_null(true)
            .with_schema(Arc::new(self.lazyframe_domain.schema()))
            .with_delimiter(self.delimiter as u8)
            .has_header(self.has_header)
            .with_skip_rows(self.skip_rows)
            .with_comment_char(self.comment_char.map(|v| v as u8))
            .with_quote_char(self.quote_char.map(|v| v as u8))
            .with_end_of_line_char(self.eol_char as u8)
            .with_null_values(self.null_values.clone())
    }
}

impl Domain for CsvDomain {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.lazyframe_domain
            .member(&self.new_reader(val.clone()).finish()?)
    }
}

impl<D: DatasetMetric> MetricSpace for (CsvDomain, D) {
    fn check(&self) -> bool {
        true
    }
}
