use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation, Downcast};
use crate::ffi::util::Type;
use crate::traits::{Float, InfCast, Integer, RoundCast};
use crate::transformations::{
    make_lipschitz_sized_proportion_ci_mean, make_lipschitz_sized_proportion_ci_variance,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_lipschitz_sized_proportion_ci_mean(
    strat_sizes: *const AnyObject,
    sample_sizes: *const AnyObject,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA: Integer, TOA: Float>(
        strat_sizes: Vec<usize>,
        sample_sizes: Vec<usize>,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TOA: RoundCast<TIA> + InfCast<TIA>,
    {
        make_lipschitz_sized_proportion_ci_mean::<TIA, TOA>(strat_sizes, sample_sizes).into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let sample_sizes = try_!(try_as_ref!(sample_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    dispatch!(monomorphize, [
        (TIA, @integers),
        (TOA, @floats)
    ], (strat_sizes, sample_sizes))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_lipschitz_sized_proportion_ci_variance(
    strat_sizes: *const AnyObject,
    sample_sizes: *const AnyObject,
    mean_scale: *const c_void,
    TIA: *const c_char,
    TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA: Integer, TOA: Float>(
        strat_sizes: Vec<usize>,
        sample_sizes: Vec<usize>,
        mean_scale: *const c_void,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TOA: RoundCast<TIA> + InfCast<TIA>,
    {
        let mean_scale = *try_as_ref!(mean_scale as *const TOA);
        make_lipschitz_sized_proportion_ci_variance::<TIA, TOA>(
            strat_sizes,
            sample_sizes,
            mean_scale,
        )
        .into_any()
    }
    let strat_sizes = try_!(try_as_ref!(strat_sizes).downcast_ref::<Vec<usize>>()).clone();
    let sample_sizes = try_!(try_as_ref!(sample_sizes).downcast_ref::<Vec<usize>>()).clone();
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));
    dispatch!(monomorphize, [
        (TIA, @integers),
        (TOA, @floats)
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
        let transformation = Result::from(opendp_transformations__make_lipschitz_sized_proportion_ci_mean(
            util::into_raw(AnyObject::new(vec![1usize, 1usize, 1usize])),
            util::into_raw(AnyObject::new(vec![1usize, 1usize, 1usize])),
            "i32".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1, 0, 1]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0 / 3.0);

        let d_in = AnyObject::new_raw(1i32);
        let res = core::opendp_core__transformation_map(&transformation, d_in);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0 / 3.0);
        Ok(())
    }

    #[test]
    fn test_lipschitz_sized_proportion_ci_variance_ffi() -> Fallible<()> {
        let transformation =
            Result::from(opendp_transformations__make_lipschitz_sized_proportion_ci_variance(
                util::into_raw(AnyObject::new(vec![10usize; 3])),
                util::into_raw(AnyObject::new(vec![5usize; 3])),
                util::into_raw(1.0) as *const c_void,
                "i32".to_char_p(),
                "f64".to_char_p(),
            ))?;
        let arg = AnyObject::new_raw(vec![1, 0, 1]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0044444444444445);

        let d_in = AnyObject::new_raw(1i32);
        let res = core::opendp_core__transformation_map(&transformation, d_in);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 0.002777777777777778);
        Ok(())
    }
}
