use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    combinators::{Invokable, OdometerAnswer, OdometerQuery},
    core::{FfiResult, Odometer, PrivacyMap},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyOdometer,
            AnyOdometerQueryable, Downcast, IntoAnyOdometerOutExt,
        },
        util::Type,
    },
    interactive::{PolyQueryable, Queryable},
};

impl Invokable<AnyDomain, AnyMetric, AnyMeasure> for AnyOdometer {
    type Output = AnyOdometerQueryable;
    fn invoke_wrap_and_map(
        &self,
        value: &AnyObject,
        wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    ) -> Fallible<(Self::Output, PrivacyMap<AnyMetric, AnyMeasure>)> {
        // wrap the child odometer to send ChildChange queries
        let answer = self
            .invoke_wrap(value, wrapper)?
            .downcast::<AnyOdometerQueryable>()?;

        let map = PrivacyMap::new_fallible(enclose!(answer, move |d_in: &AnyObject| answer
            .clone()
            .eval_map(d_in.clone())));
        Ok((answer, map))
    }

    fn one_time_privacy_map(&self) -> Option<PrivacyMap<AnyMetric, AnyMeasure>> {
        None
    }

    fn input_domain(&self) -> AnyDomain {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> AnyMetric {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> AnyMeasure {
        self.output_measure.clone()
    }
}

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

    // 1. Odometer<AnyDomain, Queryable<OdometerQuery<Q, AnyObject>, OdometerAnswer<Q::Output, AnyObject>>, AnyMetric, AnyMeasure>
    //   -> into_any_QA() ->
    // 2. Odometer<AnyDomain, Queryable<OdometerQuery<AnyObject, AnyObject>, OdometerAnswer<AnyObject, AnyObject>>, AnyMetric, AnyMeasure>
    //   -> into_any_out() ->
    // 3. Odometer<AnyDomain, AnyObject, AnyMetric, AnyObject>
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

    // match Q.id {
    //     x if x == std::any::TypeId::of::<AnyMeasurement>() => {
    //         make_sequential_odometer::<AnyMeasurement>(input_domain, input_metric, output_measure)
    //     }
    //     x if x == std::any::TypeId::of::<AnyOdometer>() => {
    //         make_sequential_odometer::<AnyOdometer>(input_domain, input_metric, output_measure)
    //     }
    //     _ => panic!("Type not supported"),
    // }
    // .into()

    dispatch!(
        make_sequential_odometer,
        [(Q, [AnyMeasurement, AnyOdometer])],
        (input_domain, input_metric, output_measure)
    )
    .into()
}
