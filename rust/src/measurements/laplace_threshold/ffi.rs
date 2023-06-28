use std::os::raw::{c_long, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{MapDomain, AtomDomain};
use crate::err;
use crate::ffi::any::{AnyMeasurement, AnyMetric, AnyDomain, Downcast, AnyObject};
use crate::ffi::util::{Type, TypeContents, self};
use crate::measurements::make_base_laplace_threshold;
use crate::metrics::L1Distance;
use crate::traits::samplers::SampleDiscreteLaplaceZ2k;
use crate::traits::{ExactIntCast, Float, Hashable};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_laplace_threshold(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    threshold: *const c_void,
    other: *const AnyObject,
    k: c_long,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TK, TV>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        threshold: *const c_void,
        other: *const AnyObject,
        k: i32,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        TK: Hashable,
        TV: Float + SampleDiscreteLaplaceZ2k,
        i32: ExactIntCast<TV::Bits>
    {
        let input_domain = try_!(input_domain.downcast_ref::<MapDomain<AtomDomain<TK>, AtomDomain<TV>>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<L1Distance<TV>>()).clone();
        let scale = *try_as_ref!(scale as *const TV);
        let threshold = *try_as_ref!(threshold as *const TV);
        let other = if let Some(other) = util::as_ref(other) {
            Some(try_!(other.downcast_ref::<TK>()).clone())
        } else {
            None
        };
        make_base_laplace_threshold::<TK, TV>(input_domain, input_metric, scale, threshold, other, Some(k)).into_any()
    }
    let k = k as i32;

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TypeContents::GENERIC {name, args} = &input_domain.carrier_type.contents else {
        return err!(FFI, "Generic type {:?} not supported", input_domain.type_.descriptor).into();
    };
    if name.starts_with("MapDomain") || args.len() != 2 {
        return err!(
            FFI,
            "Generic type {:?} not supported",
            input_domain.carrier_type.descriptor
        )
        .into();
    }
    let TK = try_!(Type::of_id(&args[0]));
    let TV = try_!(Type::of_id(&args[1]));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TV, @floats)
    ], (input_domain, input_metric, scale, threshold, other, k))
}
