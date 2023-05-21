use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    combinators::{Invokable, OdometerAnswer, OdometerQuery},
    core::{FfiResult, Odometer},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyOdometer},
        util::{AnyMeasurementPtr, AnyOdometerPtr, Type},
    },
    interactive::Queryable,
};

#[bootstrap(
    generics(Q(example = "$get_atom(get_type(output_measure))")),
    features("contrib")
)]
/// Construct a sequential odometer queryable that interactively composes odometers or interactive measurements.
///
/// # Arguments
/// * `input_domain` - indicates the space of valid input datasets
/// * `input_metric` - how distances are measured between members of the input domain
/// * `output_measure` - how privacy is measured
///
/// # Generics
/// * `Q` - either `Odometer` or `Measurement`
fn make_sequential_odometer<
    Q: 'static + Invokable<AnyDomain, AnyMetric, AnyMeasure, Output = AnyObject> + Clone,
>(
    input_domain: AnyDomain,
    input_metric: AnyMetric,
    output_measure: AnyMeasure,
) -> Fallible<AnyOdometer> {
    let compositor: Odometer<
        _,
        Queryable<OdometerQuery<Q, AnyObject>, OdometerAnswer<AnyObject, AnyObject>>,
        _,
        _,
    > = super::make_sequential_odometer(input_domain, input_metric, output_measure)?;

    // 1.   Odometer<AnyDomain, Queryable<OdometerQuery<Q, AnyObject>, OdometerAnswer<Q::Output, AnyObject>>, AnyMetric, AnyMeasure>
    //          -> into_any_QA() ->
    // 2.   Odometer<AnyDomain, Queryable<OdometerQuery<AnyObject, AnyObject>, OdometerAnswer<AnyObject, AnyObject>>, AnyMetric, AnyMeasure>
    //    = Odometer<AnyDomain, Queryable<AnyOdometerQuery,                    AnyOdometerAnswer                   >, AnyMetric, AnyMeasure>
    //          -> into_any_queryable() ->
    // 3.   Odometer<AnyDomain, AnyQueryable, AnyMetric, AnyMeasure>
    //          -> into_any_out() ->
    // 4.   Odometer<AnyDomain, AnyObject, AnyMetric, AnyObject>
    //    = AnyOdometer
    Ok(compositor.into_any_Q().into_any_queryable().into_any_out())
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

    trait OperationPtr {
        type Operation;
    }
    impl OperationPtr for AnyMeasurementPtr {
        type Operation = AnyMeasurement;
    }
    impl OperationPtr for AnyOdometerPtr {
        type Operation = AnyOdometer;
    }

    fn monomorphize<Q: OperationPtr>(
        input_domain: AnyDomain,
        input_metric: AnyMetric,
        output_measure: AnyMeasure,
    ) -> Fallible<AnyOdometer>
    where
        Q::Operation:
            'static + Invokable<AnyDomain, AnyMetric, AnyMeasure, Output = AnyObject> + Clone,
    {
        make_sequential_odometer::<Q::Operation>(input_domain, input_metric, output_measure)
    }

    dispatch!(
        monomorphize,
        [(Q, [AnyMeasurementPtr, AnyOdometerPtr])],
        (input_domain, input_metric, output_measure)
    )
    .into()
}
