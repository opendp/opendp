use std::ffi::c_char;
use std::os::raw::c_void;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure};
use crate::domains::{AtomDomain, MapDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::{Type, TypeContents, as_ref};
use crate::measurements::nature::Nature;
use crate::measurements::{MakeNoiseThreshold, make_laplace_threshold};
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::{AbsoluteDistance, L01InfDistance};
use crate::traits::{Hashable, Number};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_laplace_threshold(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    threshold: *const c_void,
    k: *const i32,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MO: 'static + Measure, TK, TV, QI>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        threshold: *const c_void,
        k: Option<i32>,
    ) -> Fallible<AnyMeasurement>
    where
        TK: Hashable,
        TV: Number + Nature,
        QI: Number,
        <TV as Nature>::RV<1>: MakeNoiseThreshold<
                MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
                L01InfDistance<AbsoluteDistance<QI>>,
                MO,
                Threshold = TV,
            >,
    {
        let input_domain = input_domain
            .downcast_ref::<MapDomain<AtomDomain<TK>, AtomDomain<TV>>>()?
            .clone();
        let input_metric = input_metric
            .downcast_ref::<L01InfDistance<AbsoluteDistance<QI>>>()?
            .clone();
        let threshold = *try_as_ref!(threshold as *const TV);
        make_laplace_threshold(input_domain, input_metric, scale, threshold, k).into_any()
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
    let MO = try_!(Type::try_from(MO));
    let TK = try_!(Type::of_id(&args[0]));
    let TV = try_!(Type::of_id(&args[1]));
    let QI = try_!(input_metric.type_.get_atom());
    let k = as_ref(k as *const i32).map(Clone::clone);

    dispatch!(monomorphize, [
        (MO, [Approximate<MaxDivergence>]),
        (TK, @hashable),
        (TV, @numbers),
        (QI, @numbers)
    ], (input_domain, input_metric, scale, threshold, k))
    .into()
}
