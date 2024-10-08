use std::os::raw::c_void;

use dashu::base::ConversionError;
use dashu::float::FBig;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::domains::{AtomDomain, MapDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::{Type, TypeContents};
use crate::measurements::make_laplace_threshold;
use crate::metrics::L1Distance;
use crate::traits::{Float, Hashable, InfCast, RoundCast};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_laplace_threshold(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    threshold: *const c_void,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TK, TV>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        threshold: *const c_void,
    ) -> Fallible<AnyMeasurement>
    where
        TK: Hashable,
        TV: Float + RoundCast<FBig>,

        FBig: TryFrom<TV, Error = ConversionError> + TryFrom<f64>,
        f64: InfCast<TV>,
    {
        let input_domain = input_domain
            .downcast_ref::<MapDomain<AtomDomain<TK>, AtomDomain<TV>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<L1Distance<TV>>()?.clone();
        let threshold = *try_as_ref!(threshold as *const TV);
        make_laplace_threshold::<TK, TV>(input_domain, input_metric, scale, threshold).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TypeContents::GENERIC { name, args } = &input_domain.carrier_type.contents else {
        return err!(
            FFI,
            "Generic type {:?} not supported",
            input_domain.type_.descriptor
        )
        .into();
    };
    if !name.starts_with("HashMap") || args.len() != 2 {
        return err!(
            FFI,
            "Domain not supported: {:?}. Must be MapDomain<AtomDomain<TK>, AtomDomain<TV>>",
            input_domain.carrier_type.descriptor
        )
        .into();
    }
    let TK = try_!(Type::of_id(&args[0]));
    let TV = try_!(Type::of_id(&args[1]));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TV, @floats)
    ], (input_domain, input_metric, scale, threshold))
    .into()
}
