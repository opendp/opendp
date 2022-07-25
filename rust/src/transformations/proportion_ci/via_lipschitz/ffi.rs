use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::Float;
use crate::trans::{
    make_lipschitz_sized_proportion_ci_mean, make_lipschitz_sized_proportion_ci_variance,
};

#[no_mangle]
pub extern "C" fn opendp_trans__make_lipschitz_sized_proportion_ci_mean(
    strat_sizes: *const AnyObject,
    sample_sizes: *const AnyObject,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA: Float>(
        strat_sizes: Vec<usize>,
        sample_sizes: Vec<usize>,
    ) -> FfiResult<*mut AnyTransformation> {
        make_lipschitz_sized_proportion_ci_mean::<TA>(strat_sizes, sample_sizes).into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let sample_sizes = try_!(try_as_ref!(sample_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (TA, @floats)
    ], (strat_sizes, sample_sizes))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_lipschitz_sized_proportion_ci_variance(
    strat_sizes: *const AnyObject,
    sample_sizes: *const AnyObject,
    mean_scale: *const c_void,
    TA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TA: Float>(
        strat_sizes: Vec<usize>,
        sample_sizes: Vec<usize>,
        mean_scale: *const c_void,
    ) -> FfiResult<*mut AnyTransformation> {
        let mean_scale = *try_as_ref!(mean_scale as *const TA);
        make_lipschitz_sized_proportion_ci_variance::<TA>(strat_sizes, sample_sizes, mean_scale)
            .into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let sample_sizes = try_!(try_as_ref!(sample_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TA = try_!(Type::try_from(TA));
    dispatch!(monomorphize, [
        (TA, @floats)
    ], (strat_sizes, sample_sizes, mean_scale))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_lipschitz_sized_proportion_ci_mean_ffi() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_lipschitz_sized_proportion_ci_mean(
            util::into_raw(AnyObject::new(vec![1usize, 1usize, 1usize])),
            util::into_raw(AnyObject::new(vec![1usize, 1usize, 1usize])),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 0.0, 1.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0 / 3.0);
        Ok(())
    }
}
