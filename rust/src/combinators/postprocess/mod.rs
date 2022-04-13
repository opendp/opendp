use crate::{
    core::{Domain, Metric, StabilityRelation, Transformation, Function},
    dist::AgnosticMetric,
    error::Fallible,
};

pub fn make_erase_relation<DI, DO, MI, MO>(
    trans: Transformation<DI, DO, MI, MO>,
) -> Fallible<Transformation<DI, DO, AgnosticMetric, AgnosticMetric>>
where
    DI: Domain,
    DO: Domain,
    MI: Metric,
    MO: Metric,
{
    Ok(Transformation {
        input_domain: trans.input_domain,
        output_domain: trans.output_domain,
        function: trans.function,
        input_metric: AgnosticMetric::default(),
        output_metric: AgnosticMetric::default(),
        stability_relation: StabilityRelation::new_all(|_d_in, _d_out| Ok(true), None::<fn(&_)->_>, None::<fn(&_)->_>),
    })
}

/// Constructs a [`Transformation`] representing an arbitrary postprocessing transformation.
pub(crate) fn make_postprocess_trans<'a, DI, DO>(
    input_domain: DI,
    output_domain: DO,
    function: impl 'static + Fn(&DI::Carrier) -> Fallible<DO::Carrier>
) -> Fallible<Transformation<DI, DO, AgnosticMetric, AgnosticMetric>>
    where DI: Domain, DO: Domain {
    Ok(Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(function),
        AgnosticMetric::default(),
        AgnosticMetric::default(),
        StabilityRelation::new_all(
            |_d_in: &(), _d_out: &()| Ok(true),
            Some(|_d_in: &()| Ok(())),
            Some(|_d_out: &()| Ok(())),
        )))
}
