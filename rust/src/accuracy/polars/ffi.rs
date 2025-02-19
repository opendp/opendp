use crate::core::{FfiResult, Function};
use crate::ffi::any::{AnyMeasurement, AnyObject, AnyQueryable, Downcast};
use crate::ffi::util;
use crate::polars::{ExtractLazyFrame, OnceFrame};
use crate::{core::Measurement, domains::LazyFrameDomain};
use polars::prelude::LazyFrame;

use super::summarize_polars_measurement;

#[no_mangle]
pub extern "C" fn opendp_accuracy__summarize_polars_measurement(
    measurement: *const AnyMeasurement,
    alpha: *const AnyObject,
) -> FfiResult<*mut AnyObject> {
    let m_untyped = try_as_ref!(measurement);
    let f_untyped = m_untyped.function.clone();

    let alpha = if let Some(param) = util::as_ref(alpha) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    let m_typed = try_!(Measurement::new(
        try_!(m_untyped.input_domain.downcast_ref::<LazyFrameDomain>()).clone(),
        Function::new_fallible(move |arg: &LazyFrame| {
            let mut qbl = f_untyped
                .eval(&AnyObject::new(arg.clone()))?
                .downcast::<AnyQueryable>()?;
            let lf: LazyFrame = qbl.eval_internal(&ExtractLazyFrame)?;
            Ok(OnceFrame::from(lf))
        }),
        m_untyped.input_metric.clone(),
        m_untyped.output_measure.clone(),
        m_untyped.privacy_map.clone()
    ));

    summarize_polars_measurement(m_typed, alpha)
        .map(AnyObject::new)
        .into()
}
