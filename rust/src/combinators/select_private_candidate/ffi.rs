use core::f64;

use crate::{
    core::{FfiResult, Function, Measurement, PrivacyMap},
    error::Fallible,
    ffi::{
        any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast},
        util::Type,
    },
    measures::MaxDivergence,
    traits::RoundCast,
};

fn to_f64<T: 'static>(obj: AnyObject) -> Fallible<f64>
where
    f64: RoundCast<T>,
{
    f64::round_cast(obj.downcast::<T>()?)
}

fn make_select_private_candidate(
    measurement: &AnyMeasurement,
    stop_probability: f64,
    threshold: f64,
) -> Fallible<AnyMeasurement> {
    let function = measurement.function.clone();
    let privacy_map = measurement.privacy_map.clone();
    let measurement = Measurement::new(
        measurement.input_domain.clone(),
        Function::new_fallible(move |arg: &AnyObject| {
            let release = function.eval(arg)?;

            // for usability and to ensure the measurement always returns data with the right type,
            // apply a postprocessor that makes the combinator more forgiving about the form of the input type

            Ok(if release.type_ == Type::of::<(f64, AnyObject)>() {
                release.downcast::<(f64, AnyObject)>()?
            } else if let Ok(val) = release.downcast::<Vec<AnyObject>>() {
                if let Ok([score, value]) = <[AnyObject; 2]>::try_from(val) {
                    let score = dispatch!(to_f64, [(score.type_, @numbers)], (score));
                    (score.unwrap_or(f64::NAN), value)
                } else {
                    (f64::NAN, AnyObject::new(()))
                }
            } else {
                (f64::NAN, AnyObject::new(()))
            })
        }),
        measurement.input_metric.clone(),
        measurement
            .output_measure
            .downcast_ref::<MaxDivergence>()?
            .clone(),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| privacy_map.eval(d_in)?.downcast()),
    )?;

    let m = super::make_select_private_candidate(measurement, stop_probability, threshold)?;

    let privacy_map = m.privacy_map.clone();
    let function = m.function.clone();

    Measurement::new(
        m.input_domain.clone(),
        Function::new_fallible(move |arg: &AnyObject| function.eval(arg).map(AnyObject::new)),
        m.input_metric.clone(),
        AnyMeasure::new(m.output_measure.clone()),
        PrivacyMap::new_fallible(move |d_in: &AnyObject| {
            privacy_map.eval(d_in).map(AnyObject::new)
        }),
    )
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_select_private_candidate(
    measurement: *const AnyMeasurement,
    stop_probability: f64,
    threshold: f64,
) -> FfiResult<*mut AnyMeasurement> {
    FfiResult::from(make_select_private_candidate(
        try_as_ref!(measurement),
        stop_probability,
        threshold,
    ))
}
