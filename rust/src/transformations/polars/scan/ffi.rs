use std::ffi::{c_char, c_uchar, c_uint};

use opendp_derive::bootstrap;
use polars::prelude::{
    CsvEncoding, LazyCsvReader, LazyFileListReader, NullValues, ScanArgsParquet,
};

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace, Transformation},
    domains::{CsvDomain, DatasetMetric, LazyFrameDomain, ParquetDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
        util::{self, c_bool},
    },
    metrics::{InsertDeleteDistance, SymmetricDistance},
};

#[bootstrap(
    features("contrib"),
    arguments(
        lazy_frame_domain(c_type = "AnyDomain *", rust_type = b"null"),
        input_metric(c_type = "AnyMetric *"),
        delimiter(default = ","),
        has_header(default = true),
        ignore_errors(default = false),
        skip_rows(default = 0),
        // n_rows(default = b"null"),
        cache(default = true),
        low_memory(default = false),
        comment_char(default = b"null"),
        quote_char(default = "\\\""),
        eol_char(default = "\\n"),
        null_value(default = b"null"),
        missing_is_null(default = true),
        rechunk(default = true),
        skip_rows_after_header(default = 0),
        lossy_utf8(default = false),
    ),

    // generics(M(suppress))
)]
/// Parse a path to a CSV into a LazyFrame.
///
/// # Arguments
/// * `lazy_frame_domain` - The domain of the LazyFrame to be constructed
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `delimiter` - Set the CSV file's column delimiter as a byte character
/// * `has_header` - Set whether the CSV file has headers
/// * `ignore_errors` - Continue with next batch when a ParserError is encountered.
/// * `skip_rows` - Skip the first `n` rows during parsing. The header will be parsed at row `n`.
// /// * `n_rows` - Try to stop parsing when `n` rows are parsed. During multithreaded parsing the upper bound `n` cannot be guaranteed.
/// * `cache` - Cache the DataFrame after reading.
/// * `low_memory` - Reduce memory usage at the expense of performance
/// * `comment_char` - Set the comment character. Lines starting with this character will be ignored.
/// * `quote_char` - Set the `char` used as quote char. The default is `"`. If set to `[None]` quoting is disabled.
/// * `eol_char` - Set the `char` used as end of line. The default is `\n`.
/// * `null_value` - Set value that will be interpreted as missing/ null.
/// * `missing_is_null` - Treat missing fields as null.
/// * `rechunk` - Rechunk the memory to contiguous chunks when parsing is done.
/// * `skip_rows_after_header` - Skip this number of rows after the header location.
/// * `lossy_utf8` - If enabled, unknown bytes in Utf8 encoding are replaced with �
///
/// # Generics
/// * `M` - One of the dataset metrics
fn make_scan_csv<M: DatasetMetric>(
    lazy_frame_domain: LazyFrameDomain,
    input_metric: M,
    delimiter: char,
    has_header: bool,
    ignore_errors: bool,
    skip_rows: usize,
    // n_rows: Option<usize>,
    cache: bool,
    low_memory: bool,
    comment_char: Option<char>,
    quote_char: Option<char>,
    eol_char: char,
    null_value: Option<String>,
    missing_is_null: bool,
    rechunk: bool,
    skip_rows_after_header: usize,
    lossy_utf8: bool,
) -> Fallible<Transformation<CsvDomain, LazyFrameDomain, M, M>>
where
    (CsvDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let reader = LazyCsvReader::new("")
        .with_delimiter(delimiter as u8)
        .has_header(has_header)
        .with_ignore_errors(ignore_errors)
        .with_skip_rows(skip_rows)
        // .with_n_rows(n_rows)
        .with_cache(cache)
        .low_memory(low_memory)
        .with_comment_char(comment_char.map(|v| v as u8))
        .with_quote_char(quote_char.map(|v| v as u8))
        .with_end_of_line_char(eol_char as u8)
        .with_null_values(null_value.map(NullValues::AllColumnsSingle))
        .with_missing_is_null(missing_is_null)
        .with_rechunk(rechunk)
        .with_skip_rows_after_header(skip_rows_after_header)
        .with_encoding(if lossy_utf8 {
            CsvEncoding::LossyUtf8
        } else {
            CsvEncoding::Utf8
        });

    let input_domain = CsvDomain {
        lazy_frame_domain,
        reader,
    };

    super::make_scan_csv(input_domain, input_metric)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_scan_csv(
    lazy_frame_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    delimiter: c_uchar,
    has_header: c_bool,
    ignore_errors: c_bool,
    skip_rows: c_uint,
    // n_rows: *const c_uint,
    cache: c_bool,
    low_memory: c_bool,
    comment_char: *const c_uchar,
    quote_char: *const c_uchar,
    eol_char: c_uchar,
    null_value: *const c_char,
    missing_is_null: c_bool,
    rechunk: c_bool,
    skip_rows_after_header: c_uint,
    lossy_utf8: c_bool,
) -> FfiResult<*mut AnyTransformation> {
    let lazy_frame_domain =
        try_!(try_as_ref!(lazy_frame_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let delimiter: char = delimiter as char;
    let has_header: bool = util::to_bool(has_header);
    let ignore_errors: bool = util::to_bool(ignore_errors);
    let skip_rows: usize = skip_rows as usize;
    // let n_rows: Option<usize> = util::as_ref(n_rows).map(|v| *v as usize);
    let cache: bool = util::to_bool(cache);
    let low_memory: bool = util::to_bool(low_memory);
    let comment_char: Option<char> = util::as_ref(comment_char).map(|v| *v as char);
    let quote_char: Option<char> = util::as_ref(quote_char).map(|v| *v as char);
    let eol_char: char = eol_char as char;
    let null_value: Option<String> = try_!(util::to_option_str(null_value)).map(|v| v.to_string());
    let missing_is_null: bool = util::to_bool(missing_is_null);
    let rechunk: bool = util::to_bool(rechunk);
    let skip_rows_after_header: usize = skip_rows_after_header as usize;
    let lossy_utf8: bool = util::to_bool(lossy_utf8);

    fn monomorphize<M: 'static + DatasetMetric>(
        lazy_frame_domain: LazyFrameDomain,
        input_metric: &AnyMetric,
        delimiter: char,
        has_header: bool,
        ignore_errors: bool,
        skip_rows: usize,
        // n_rows: Option<usize>,
        cache: bool,
        low_memory: bool,
        comment_char: Option<char>,
        quote_char: Option<char>,
        eol_char: char,
        null_value: Option<String>,
        missing_is_null: bool,
        rechunk: bool,
        skip_rows_after_header: usize,
        lossy_utf8: bool,
    ) -> FfiResult<*mut AnyTransformation>
    where
        (CsvDomain, M): MetricSpace,
        (LazyFrameDomain, M): MetricSpace,
    {
        let input_metric: M = try_!(input_metric.downcast_ref::<M>()).clone();
        make_scan_csv(
            lazy_frame_domain,
            input_metric,
            delimiter,
            has_header,
            ignore_errors,
            skip_rows,
            // n_rows,
            cache,
            low_memory,
            comment_char,
            quote_char,
            eol_char,
            null_value,
            missing_is_null,
            rechunk,
            skip_rows_after_header,
            lossy_utf8,
        )
        .into_any()
    }
    let M = input_metric.type_.clone();

    dispatch!(
        monomorphize,
        [(M, [SymmetricDistance, InsertDeleteDistance])],
        (
            lazy_frame_domain,
            input_metric,
            delimiter,
            has_header,
            ignore_errors,
            skip_rows,
            // n_rows,
            cache,
            low_memory,
            comment_char,
            quote_char,
            eol_char,
            null_value,
            missing_is_null,
            rechunk,
            skip_rows_after_header,
            lossy_utf8
        )
    )
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sink_csv(
    lazy_frame_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_path: *mut c_char,
) -> FfiResult<*mut AnyTransformation> {
    let lazy_frame_domain =
        try_!(try_as_ref!(lazy_frame_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let output_path = try_!(util::to_str(output_path)).to_string();

    fn monomorphize<MI: 'static + DatasetMetric>(
        lazy_frame_domain: LazyFrameDomain,
        input_metric: &AnyMetric,
        output_path: String,
    ) -> FfiResult<*mut AnyTransformation>
    where
        (LazyFrameDomain, MI): MetricSpace,
        (CsvDomain, MI): MetricSpace,
    {
        let input_metric: MI = try_!(input_metric.downcast_ref::<MI>()).clone();
        super::make_sink_csv(lazy_frame_domain, input_metric, output_path).into_any()
    }
    let M = input_metric.type_.clone();

    dispatch!(
        monomorphize,
        [(M, [SymmetricDistance, InsertDeleteDistance])],
        (lazy_frame_domain, input_metric, output_path)
    )
}

#[bootstrap(
    features("contrib"),
    arguments(
        lazy_frame_domain(c_type = "AnyDomain *", rust_type = b"null"),
        input_metric(c_type = "AnyMetric *"),
        delimiter(default = ","),
        has_header(default = true),
        ignore_errors(default = false),
        skip_rows(default = 0),
        n_rows(default = b"null"),
        cache(default = true),
        low_memory(default = false),
        comment_char(default = b"null"),
        quote_char(default = "\\\""),
        eol_char(default = "\\n"),
        null_value(default = b"null"),
        missing_is_null(default = true),
        rechunk(default = true),
        skip_rows_after_header(default = 0),
        lossy_utf8(default = false),
    ),

    // generics(M(suppress))
)]
/// Parse a path to a CSV into a LazyFrame.
///
/// # Arguments
/// * `lazy_frame_domain` - The domain of the LazyFrame to be constructed
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `cache` - Cache the result after reading.
/// * `rechunk` - In case of reading multiple files via a glob pattern rechunk the final DataFrame into contiguous memory chunks.
/// * `low_memory` - Reduce memory pressure at the expense of performance.
/// * `use_statistics` - Use statistics from parquet to determine if pages can be skipped from reading.
///
/// # Generics
/// * `M` - A dataset metric
fn make_scan_parquet<M: DatasetMetric>(
    lazy_frame_domain: LazyFrameDomain,
    input_metric: M,
    cache: bool,
    rechunk: bool,
    low_memory: bool,
    use_statistics: bool,
) -> Fallible<Transformation<ParquetDomain, LazyFrameDomain, M, M>>
where
    (ParquetDomain, M): MetricSpace,
    (LazyFrameDomain, M): MetricSpace,
{
    let input_domain = ParquetDomain {
        lazy_frame_domain,
        scan_args_parquet: ScanArgsParquet {
            cache,
            rechunk,
            low_memory,
            use_statistics,
            ..Default::default()
        },
    };

    super::make_scan_parquet(input_domain, input_metric)
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_scan_parquet(
    lazy_frame_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    cache: c_bool,
    rechunk: c_bool,
    low_memory: c_bool,
    use_statistics: c_bool,
) -> FfiResult<*mut AnyTransformation> {
    let lazy_frame_domain =
        try_!(try_as_ref!(lazy_frame_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let cache: bool = util::to_bool(cache);
    let rechunk: bool = util::to_bool(rechunk);
    let low_memory: bool = util::to_bool(low_memory);
    let use_statistics: bool = util::to_bool(use_statistics);

    fn monomorphize<M: 'static + DatasetMetric>(
        lazy_frame_domain: LazyFrameDomain,
        input_metric: &AnyMetric,
        cache: bool,
        rechunk: bool,
        low_memory: bool,
        use_statistics: bool,
    ) -> FfiResult<*mut AnyTransformation>
    where
        (ParquetDomain, M): MetricSpace,
        (LazyFrameDomain, M): MetricSpace,
    {
        let input_metric: M = try_!(input_metric.downcast_ref::<M>()).clone();
        make_scan_parquet(
            lazy_frame_domain,
            input_metric,
            cache,
            rechunk,
            low_memory,
            use_statistics,
        )
        .into_any()
    }
    let M = input_metric.type_.clone();

    dispatch!(
        monomorphize,
        [(M, [SymmetricDistance, InsertDeleteDistance])],
        (
            lazy_frame_domain,
            input_metric,
            cache,
            rechunk,
            low_memory,
            use_statistics
        )
    )
}
