use std::os::raw::c_char;

use crate::util;

pub mod dataframe;
pub mod manipulation;
pub mod sum;
pub mod count;
pub mod mean;
pub mod variance;

#[no_mangle]
pub extern "C" fn opendp_trans__bootstrap() -> *const c_char {
    let spec =
        r#"{
"functions": [
    { "name": "make_identity", "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_split_lines", "args": [ ["const char *", "selector"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_parse_series", "args": [ ["const char *", "selector"], ["bool", "impute"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_split_records", "args": [ ["const char *", "selector"], ["const char *", "separator"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_create_dataframe", "args": [ ["const char *", "selector"], ["FfiObject *", "col_names"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_split_dataframe", "args": [ ["const char *", "selector"], ["const char *", "separator"], ["FfiObject *", "col_names"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_parse_column", "args": [ ["const char *", "selector"], ["void *", "key"], ["bool", "impute"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_select_column", "args": [ ["const char *", "selector"], ["void *", "key"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_clamp_vec", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_clamp_scalar", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_cast_vec", "args": [ ["const char *", "selector"] ], "ret": "FfiResult<FfiTransformation *>" },

    { "name": "make_bounded_covariance", "args": [ ["const char *", "selector"], ["FfiObject *", "lower"], ["FfiObject *", "upper"], ["unsigned int", "length"], ["unsigned int", "ddof"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_bounded_mean", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"], ["unsigned int", "length"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_bounded_sum", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_bounded_sum_n", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"], ["unsigned int", "n"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_bounded_variance", "args": [ ["const char *", "selector"], ["void *", "lower"], ["void *", "upper"], ["unsigned int", "length"], ["unsigned int", "ddof"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_count", "args": [ ["const char *", "selector"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_count_by", "args": [ ["const char *", "selector"], ["unsigned int", "n"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_count_by_categories", "args": [ ["const char *", "selector"], ["FfiObject *", "categories"] ], "ret": "FfiResult<FfiTransformation *>" }
]
}"#;
    util::bootstrap(spec)
}
