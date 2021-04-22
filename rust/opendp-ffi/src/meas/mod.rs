pub mod gaussian;
pub mod geometric;
pub mod laplace;
pub mod stability;

use crate::util;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn opendp_meas__bootstrap() -> *const c_char {
    let spec =
        r#"{
"functions": [
    { "name": "make_base_laplace", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_laplace_vec", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_gaussian", "args": [ ["const char *", "selector"], ["void *", "scale"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_simple_geometric", "args": [ ["const char *", "selector"], ["void *", "scale"], ["void *", "min"], ["void *", "max"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_base_stability", "args": [ ["const char *", "selector"], ["unsigned int", "n"], ["void *", "scale"], ["void *", "threshold"] ], "ret": "FfiResult<FfiMeasurement *>" }
]
}"#;
    util::bootstrap(spec)
}

