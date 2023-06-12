use std::ffi::{c_uint, c_uchar};

use opendp_derive::bootstrap;

use crate::{
    core::FfiResult,
    ffi::{any::{AnyDomain, Downcast}, util::{c_bool, self}}, domains::LazyFrameDomain,
};

use super::CsvDomain;

#[no_mangle]
#[bootstrap(
    name = "csv_domain",
    features("contrib"),
    arguments(
        lazyframe_domain(c_type = "AnyDomain *", rust_type = b"null"),
        delimiter(default = ",", c_type = "char", rust_type = "char"),
        has_header(default = true, rust_type="bool"),
        skip_rows(default = 0, c_type = "unsigned int", rust_type = "usize"),
        comment_char(default = b"null", c_type = "char *", rust_type = "Option<char>"),
        quote_char(default = "\\\"", c_type = "char *", rust_type = "Option<char>"),
        eol_char(default = "\\n", c_type = "char", rust_type = "char"),
        // null_values(default = b"null"),
    )
)]
/// Parse a path to a CSV into a LazyFrame.
///
/// # Arguments
/// * `lazyframe_domain` - The domain of the LazyFrame to be constructed
/// * `delimiter` - Set the CSV file's column delimiter as a byte character
/// * `has_header` - Set whether the CSV file has headers
/// * `skip_rows` - Skip the first `n` rows during parsing. The header will be parsed at row `n`.
/// * `comment_char` - Set the comment character. Lines starting with this character will be ignored.
/// * `quote_char` - Set the `char` used as quote char. The default is `"`. If set to `[None]` quoting is disabled.
/// * `eol_char` - Set the `char` used as end of line. The default is `\\n`.
// /// * `null_values` - Set value that will be interpreted as missing/ null.
pub extern "C" fn opendp_domains__csv_domain(
    lazyframe_domain: *const AnyDomain,
    delimiter: c_uchar,
    has_header: c_bool,
    skip_rows: c_uint,
    comment_char: *const c_uchar,
    quote_char: *const c_uchar,
    eol_char: c_uchar,
) -> FfiResult<*mut AnyDomain> {
    let lazyframe_domain = try_!(try_as_ref!(lazyframe_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let domain = CsvDomain::new(lazyframe_domain)
        .with_delimiter(delimiter as char)
        .has_header(util::to_bool(has_header))
        .with_skip_rows(skip_rows as usize)
        .with_comment_char(util::as_ref(comment_char).map(|v| *v as char))
        .with_quote_char(util::as_ref(quote_char).map(|v| *v as char))
        .with_end_of_line_char(eol_char as char);
        // .with_null_values(null_values)
    FfiResult::Ok(AnyDomain::new_raw(domain))
}
