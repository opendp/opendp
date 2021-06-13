use std::os::raw::{c_char, c_void};
use crate::core::FfiResult;
use crate::any::AnyTransformation;
use opendp::trans::make_impute_constant;


// #[no_mangle]
// pub extern "C" fn opendp_trans__make_impute_constant(
//     lower: *const c_void, upper: *const c_void,
//     M: *const c_char, T: *const c_char,
// ) -> FfiResult<*mut AnyTransformation> {
//
// }