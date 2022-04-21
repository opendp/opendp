use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::dom::{AllDomain, VectorDomain};
use crate::ffi::any::{AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::{Type, self};
use crate::meas::{make_base_discrete_laplace, DiscreteLaplaceDomain};
use crate::traits::{InfAdd, InfCast, RoundCast, TotalOrd};
use crate::trans::{GreatestDifference, SameMetric};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_discrete_laplace(
    scale: *const c_void,
    bounds: *const AnyObject,
    granularity: *const c_void,
    D: *const c_char,
    I: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D, I>(
        scale: *const c_void,
        bounds: *const AnyObject,
        granularity: *const c_void,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        D: DiscreteLaplaceDomain<I>,
        I: 'static
            + InfCast<D::Atom>
            + RoundCast<D::Atom>
            + Clone
            + TotalOrd
            + GreatestDifference<D::Atom>
            + InfAdd,
        // metrics match, but associated distance types may vary
        (D::Metric, D::IntegerMetric): SameMetric<D::Metric, D::IntegerMetric>,
    {
        let scale = *try_as_ref!(scale as *const D::Atom);
        let bounds = if let Some(bounds) = util::as_ref(bounds) {
            Some(try_!(bounds.downcast_ref::<(D::Atom, D::Atom)>()).clone())
        } else {
            None
        };
        let granularity = util::as_ref(granularity as *const D::Atom).cloned();
        make_base_discrete_laplace::<D, I>(scale, bounds, granularity).into_any()
    }
    let D = try_!(Type::try_from(D));
    let I = try_!(Type::try_from(I));
    dispatch!(monomorphize, [
        (D, [AllDomain<f64>, AllDomain<f32>, VectorDomain<AllDomain<f64>>, VectorDomain<AllDomain<f32>>]),
        (I, @integers)
    ], (scale, bounds, granularity))
}
