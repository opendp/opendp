use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::path::PathBuf;

use polars::prelude::*;

use crate::domains::{Frame, FrameDomain};
use crate::{
    core::{Domain, Metric, MetricSpace},
    error::Fallible,
    metrics::{
        ChangeOneDistance, HammingDistance, InsertDeleteDistance, IntDistance, SymmetricDistance,
    },
};

#[cfg(feature = "ffi")]
mod ffi;

pub trait DatasetMetric: Metric<Distance = IntDistance> {}
impl DatasetMetric for SymmetricDistance {}
impl DatasetMetric for InsertDeleteDistance {}
impl DatasetMetric for ChangeOneDistance {}
impl DatasetMetric for HammingDistance {}

#[derive(Clone)]
pub struct CsvDomain<F: Frame> {
    pub frame_domain: FrameDomain<F>,
    pub delimiter: char,
    pub has_header: bool,
    pub skip_rows: usize,
    pub comment_char: Option<char>,
    pub quote_char: Option<char>,
    pub eol_char: char,
    pub null_values: Option<NullValues>,
    pub null_value_repr: String,
}

impl<F: Frame> CsvDomain<F> {
    pub fn new(frame_domain: FrameDomain<F>) -> Self {
        CsvDomain {
            frame_domain,
            delimiter: ',',
            has_header: true,
            skip_rows: 0,
            comment_char: None,
            quote_char: Some('"'),
            eol_char: '\n',
            null_values: None,
            null_value_repr: "None".to_string(),
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

    /// Set the file’s null value representation.
    pub fn with_null_value_repr(mut self, null_value_repr: String) -> Self {
        self.null_value_repr = null_value_repr;
        self
    }

    pub fn new_reader<'a>(&self, path: PathBuf) -> LazyCsvReader<'a> {
        LazyCsvReader::new(path)
            // parsing errors are a side-channel
            .with_ignore_errors(true)
            // fill missing columns with null
            .with_missing_is_null(true)
            .with_schema(Arc::new(self.frame_domain.schema()))
            .with_delimiter(self.delimiter as u8)
            .has_header(self.has_header)
            .with_skip_rows(self.skip_rows)
            .with_comment_char(self.comment_char.map(|v| v as u8))
            .with_quote_char(self.quote_char.map(|v| v as u8))
            .with_end_of_line_char(self.eol_char as u8)
            .with_null_values(self.null_values.clone())
    }

    pub fn new_writer(&self, path: PathBuf) -> CsvWriter<File> {
        let file = File::create(path).unwrap();

        CsvWriter::new(file)
            .with_delimiter(self.delimiter as u8)
            .has_header(self.has_header)
            .with_quoting_char(self.quote_char.map(|v| v as u8).unwrap())
            .with_null_value(self.null_value_repr.clone())
    }
}

impl<F: Frame> PartialEq for CsvDomain<F> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<F: Frame> Debug for CsvDomain<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Domain for CsvDomain<LazyFrame> {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.frame_domain
            .member(&self.new_reader(val.clone()).finish()?)
    }
}

impl Domain for CsvDomain<DataFrame> {
    type Carrier = PathBuf;

    fn member(&self, val: &Self::Carrier) -> Fallible<bool> {
        self.frame_domain
            .member(&self.new_reader(val.clone()).finish()?.collect().unwrap())
    }
}

impl<D: DatasetMetric, F: Frame> MetricSpace for (CsvDomain<F>, D) {
    fn check(&self) -> bool {
        true
    }
}
