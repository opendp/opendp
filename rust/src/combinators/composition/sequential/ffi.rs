use opendp_derive::bootstrap;

use crate::{
    core::{FfiResult, Measurement},
    error::Fallible,
    ffi::any::{
        AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast,
        IntoAnyMeasurementOutExt,
    },
    interactive::Queryable,
};

#[bootstrap(
    name = "make_sequential_composition",
    features("contrib"),
    arguments(
        d_in(rust_type = "$get_distance_type(input_metric)", c_type = "AnyObject *"),
        d_mids(rust_type = "Vec<QO>", c_type = "AnyObject *")
    ),
    derived_types(QO = "$get_distance_type(output_measure)")
)]
/// Construct a queryable that interactively composes interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
/// * `d_in` - maximum distance between adjacent input datasets
/// * `d_mids` - maximum privacy expenditure of each query
fn make_sequential_composition(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
    d_in: AnyObject,
    d_mids: Vec<AnyObject>,
) -> Fallible<AnyMeasurement> {
    let compositor: Measurement<_, Queryable<AnyMeasurement, AnyObject>, _, _> =
        super::make_sequential_composition(
            input_domain,
            input_metric,
            output_measure,
            d_in,
            d_mids,
        )?;

    Ok(compositor.into_any_Q().into_any_out())
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_sequential_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
    d_mids: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();
    let d_in = try_as_ref!(d_in).clone();
    let d_mids = try_as_ref!(d_mids);

    fn repack_vec<T: 'static + Clone>(obj: &AnyObject) -> Fallible<Vec<AnyObject>> {
        Ok(obj
            .downcast_ref::<Vec<T>>()?
            .iter()
            .map(Clone::clone)
            .map(AnyObject::new)
            .collect())
    }

    let QO = try_!(output_measure.type_.get_atom());
    let d_mids = try_!(dispatch!(repack_vec, [(QO, @numbers)], (d_mids)));

    make_sequential_composition(input_domain, input_metric, output_measure, d_in, d_mids).into()
}
