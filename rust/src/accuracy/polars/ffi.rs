use crate::core::{FfiResult, Function, Measurement};
use crate::domains::{Database, DatabaseDomain, LazyFrameDomain};
use crate::ffi::any::{AnyMeasurement, AnyObject, AnyQueryable, Downcast};
use crate::ffi::util;
use crate::polars::{ExtractLazyFrame, OnceFrame};
use polars::prelude::{DataFrame, IntoLazy, LazyFrame};

use super::{summarize_lazyframe, summarize_polars_measurement};

#[unsafe(no_mangle)]
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

    if let Ok(input_domain) = m_untyped.input_domain.downcast_ref::<DatabaseDomain>() {
        let database: Database = try_!(
            input_domain
                .0
                .iter()
                .map(|(name, domain)| {
                    Ok((
                        name.clone(),
                        DataFrame::from_rows_and_schema(&[], &domain.schema())?.lazy(),
                    ))
                })
                .collect::<crate::error::Fallible<Database>>()
        );

        let mut qbl =
            try_!(try_!(f_untyped.eval(&AnyObject::new(database))).downcast::<AnyQueryable>());
        let lf: LazyFrame = try_!(qbl.eval_internal(&ExtractLazyFrame));

        return summarize_lazyframe(&lf, alpha).map(AnyObject::new).into();
    }

    let m_typed = try_!(Measurement::new(
        try_!(m_untyped.input_domain.downcast_ref::<LazyFrameDomain>()).clone(),
        m_untyped.input_metric.clone(),
        m_untyped.output_measure.clone(),
        Function::new_fallible(move |arg: &LazyFrame| {
            let mut qbl = f_untyped
                .eval(&AnyObject::new(arg.clone()))?
                .downcast::<AnyQueryable>()?;
            let lf: LazyFrame = qbl.eval_internal(&ExtractLazyFrame)?;
            Ok(OnceFrame::from(lf))
        }),
        m_untyped.privacy_map.clone()
    ));

    summarize_polars_measurement(m_typed, alpha)
        .map(AnyObject::new)
        .into()
}
