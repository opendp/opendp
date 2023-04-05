use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    combinators::{Invokable, OdometerAnswer, OdometerQuery, OdometerQueryable},
    core::{FfiResult, Odometer},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyOdometer,
            IntoAnyOdometerOutExt,
        },
        util::Type,
    },
    interactive::Queryable,
};

#[bootstrap(name = "make_sequential_odometer", features("contrib"))]
/// Construct a sequential odometer queryable that interactively composes odometers or interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
fn make_sequential_odometer<Q: 'static + Invokable<AnyDomain, AnyMetric, AnyMeasure> + Clone>(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
) -> Fallible<AnyOdometer> {
    let compositor: Odometer<
        _,
        Queryable<OdometerQuery<Q, AnyObject>, OdometerAnswer<Q::Output, AnyObject>>,
        _,
        _,
    > = super::make_sequential_odometer(input_domain, input_metric, output_measure)?;

    Ok(compositor.into_any_QA().into_any_out())
}

#[no_mangle]
pub extern "C" fn opendp_combinators__make_sequential_odometer(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    Q: *const c_char,
) -> FfiResult<*mut AnyOdometer> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();

    let Q = try_!(Type::try_from(Q));

    fn monomorphize<Q: 'static + Invokable<AnyDomain, AnyMetric, AnyMeasure> + Clone>(
        input_domain: AnyDomain,
        input_metric: AnyMetric,
        output_measure: AnyMeasure,
    ) -> Fallible<AnyOdometer> {
        make_sequential_odometer::<Q>(input_domain, input_metric, output_measure)
    }

    match Q.id {
        x if x == std::any::TypeId::of::<AnyMeasurement>() => {
            monomorphize::<AnyMeasurement>(input_domain, input_metric, output_measure)
        }
        x if x == std::any::TypeId::of::<AnyOdometer>() => {
            // TODO: is this valid?
            monomorphize::<
                Odometer<
                    AnyDomain,
                    OdometerQueryable<AnyObject, AnyObject, AnyObject, AnyObject>,
                    AnyMetric,
                    AnyMeasure,
                >,
            >(input_domain, input_metric, output_measure)
        }
        _ => panic!("Type not supported"),
    }
    .into()

    // dispatch!(
    //     monomorphize,
    //     [(Q, [AnyMeasurement, AnyOdometer])],
    //     (input_domain, input_metric, output_measure)
    // )
    // .into()
}
