pub mod filter;
pub mod sequential;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

use std::any::Any;

use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{merge_wrappers, Answer, PolyQueryable, Query, Queryable, WrapFn},
};

pub type OdometerQueryable<Q, A, QU, AU> = Queryable<OdometerQuery<Q, QU>, OdometerAnswer<A, AU>>;

pub enum OdometerQuery<Q, U> {
    Invoke(Q),
    Map(U),
}

pub enum OdometerAnswer<A, U> {
    Invoke(A),
    Map(U),
}

// convenience eval's for invoking of mapping distances over the odometer queryable
impl<Q, A, QU: 'static, AU: 'static> Queryable<OdometerQuery<Q, QU>, OdometerAnswer<A, AU>> {
    pub fn eval_invoke(&mut self, query: Q) -> Fallible<A> {
        if let OdometerAnswer::Invoke(answer) = self.eval(&OdometerQuery::Invoke(query))? {
            Ok(answer)
        } else {
            fallible!(FailedCast, "return type is not an answer")
        }
    }
    pub(crate) fn eval_invoke_wrap(&mut self, query: Q, wrapper: Option<WrapFn>) -> Fallible<A> {
        if let OdometerAnswer::Invoke(answer) =
            self.eval_wrap(&OdometerQuery::Invoke(query), wrapper)?
        {
            Ok(answer)
        } else {
            fallible!(FailedCast, "return type is not an answer")
        }
    }
    pub fn eval_map(&mut self, d_in: QU) -> Fallible<AU> {
        if let OdometerAnswer::Map(map) = self.eval(&OdometerQuery::Map(d_in))? {
            Ok(map)
        } else {
            fallible!(FailedCast, "return type is not a privacy map")
        }
    }
}

impl<Q, QU: 'static, AU: 'static>
    Queryable<OdometerQuery<Q, QU>, OdometerAnswer<Box<dyn Any>, AU>>
{
    pub fn eval_invoke_poly<A: 'static>(&mut self, query: Q) -> Fallible<A> {
        self.eval_invoke(query)?
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "eval_poly failed to downcast to {}",
                    std::any::type_name::<A>()
                )
            })
            .map(|b| *b)
    }
}

pub trait IsInvokable<DI: Domain, MI: Metric, MO: Measure> {
    type Output;
    fn invoke_wrap_and_map(
        &self,
        value: &DI::Carrier,
        wrapper: Option<WrapFn>,
    ) -> Fallible<(Self::Output, PrivacyMap<MI, MO>)>;

    // still used to determine privacy usage after running new invokable
    fn one_time_privacy_map(&self) -> Option<PrivacyMap<MI, MO>>;

    fn input_domain(&self) -> DI;
    fn input_metric(&self) -> MI;
    fn output_measure(&self) -> MO;
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> IsInvokable<DI, MI, MO>
    for Measurement<DI, TO, MI, MO>
where
    (DI, MI): MetricSpace,
{
    type Output = TO;
    fn invoke_wrap_and_map(
        &self,
        value: &DI::Carrier,
        _wrapper: Option<WrapFn>,
    ) -> Fallible<(Self::Output, PrivacyMap<MI, MO>)> {
        Ok((self.invoke(value)?, self.privacy_map.clone()))
    }

    fn one_time_privacy_map(&self) -> Option<PrivacyMap<MI, MO>> {
        Some(self.privacy_map.clone())
    }

    fn input_domain(&self) -> DI {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> MI {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> MO {
        self.output_measure.clone()
    }
}

impl<DI: Domain + 'static, Q: 'static, A: 'static, MI: Metric + 'static, MO: Measure + 'static>
    IsInvokable<DI, MI, MO>
    for Odometer<DI, OdometerQueryable<Q, A, MI::Distance, MO::Distance>, MI, MO>
where
    MI::Distance: Clone,
    (DI, MI): MetricSpace,
{
    type Output = OdometerQueryable<Q, A, MI::Distance, MO::Distance>;
    fn invoke_wrap_and_map(
        &self,
        value: &DI::Carrier,
        wrapper: Option<WrapFn>,
    ) -> Fallible<(Self::Output, PrivacyMap<MI, MO>)> {
        // wrap the child odometer to send ChildChange queries
        let answer = self.invoke_wrap(value, wrapper)?;

        let map = PrivacyMap::new_fallible(enclose!(answer, move |d_in: &MI::Distance| answer
            .clone()
            .eval_map(d_in.clone())));
        Ok((answer, map))
    }

    fn one_time_privacy_map(&self) -> Option<PrivacyMap<MI, MO>> {
        None
    }

    fn input_domain(&self) -> DI {
        self.input_domain.clone()
    }

    fn input_metric(&self) -> MI {
        self.input_metric.clone()
    }

    fn output_measure(&self) -> MO {
        self.output_measure.clone()
    }
}

#[derive(Clone)]
pub enum Invokable<DI: Domain, TO, MI: Metric, MO: Measure> {
    Measurement(Measurement<DI, TO, MI, MO>),
    Odometer(Odometer<DI, TO, MI, MO>),
}
impl<DI: Domain, TO, MI: Metric, MO: Measure> IsInvokable<DI, MI, MO> for Invokable<DI, TO, MI, MO>
where
    Measurement<DI, TO, MI, MO>: IsInvokable<DI, MI, MO, Output = TO>,
    Odometer<DI, TO, MI, MO>: IsInvokable<DI, MI, MO, Output = TO>,
{
    type Output = TO;

    fn invoke_wrap_and_map(
        &self,
        value: &<DI as Domain>::Carrier,
        wrapper: Option<WrapFn>,
    ) -> Fallible<(Self::Output, PrivacyMap<MI, MO>)> {
        match self {
            Invokable::Measurement(meas) => meas.invoke_wrap_and_map(value, wrapper),
            Invokable::Odometer(odo) => odo.invoke_wrap_and_map(value, wrapper),
        }
    }

    fn one_time_privacy_map(&self) -> Option<PrivacyMap<MI, MO>> {
        match self {
            Invokable::Measurement(meas) => meas.one_time_privacy_map(),
            Invokable::Odometer(odo) => odo.one_time_privacy_map(),
        }
    }

    fn input_domain(&self) -> DI {
        match self {
            Invokable::Measurement(meas) => meas.input_domain.clone(),
            Invokable::Odometer(odo) => odo.input_domain.clone(),
        }
    }

    fn input_metric(&self) -> MI {
        match self {
            Invokable::Measurement(meas) => meas.input_metric.clone(),
            Invokable::Odometer(odo) => odo.input_metric.clone(),
        }
    }

    fn output_measure(&self) -> MO {
        match self {
            Invokable::Measurement(meas) => meas.output_measure.clone(),
            Invokable::Odometer(odo) => odo.output_measure.clone(),
        }
    }
}

// eval_internal(ChildChange) -> PrivacyMap
#[derive(Clone)]
struct ChildChange<MI: Metric, MO: Measure> {
    pub id: usize,
    pub new_privacy_map: PrivacyMap<MI, MO>,
}
struct GetId;

impl WrapFn {
    pub fn new_odo<MI: Metric + 'static, MO: Measure + 'static>(parent: PolyQueryable) -> Self {
        WrapFn::new(move |wrap_logic, mut inner_qbl| {
            let mut parent = parent.clone();

            Queryable::new(
                move |_self: &PolyQueryable, query: Query<dyn Any>| {
                    match query {
                        Query::External(ext, outer_wrapper) => {
                            let pending_privacy_map: PrivacyMap<MI, MO> =
                                inner_qbl.eval_internal(ext)?;

                            parent.eval_internal(&ChildChange {
                                new_privacy_map: pending_privacy_map,
                                id: inner_qbl.eval_internal(&GetId)?,
                            })?;

                            let mapper = WrapFn::new(move |inner_wrapper, qbl: PolyQueryable| {
                                (inner_wrapper.0)(WrapFn::new_odo::<MI, MO>(qbl.clone()), qbl)
                            });
                            let merged = merge_wrappers(outer_wrapper, Some(mapper));
                            Ok(Answer::External(inner_qbl.eval_wrap(ext, merged)?))
                        }
                        Query::Internal(internal_query) => {
                            // construct what this odometer's privacy map would be after after integrating this privacy map
                            if let Some(change_query) =
                                internal_query.downcast_ref::<ChildChange<MI, MO>>()
                            {
                                // pass the query through to the child, and get the new child map
                                let pending_map: PrivacyMap<MI, MO> =
                                    inner_qbl.eval_internal(change_query)?;

                                // ask permission from the parent
                                parent.eval_internal(&ChildChange {
                                    id: inner_qbl.eval_internal(&GetId)?,
                                    new_privacy_map: pending_map.clone(),
                                })?;

                                // return the pending map to the caller
                                return Ok(Answer::internal(pending_map));
                            }

                            return fallible!(FailedFunction, "unrecognized internal query");
                        }
                    }
                },
                Some(wrap_logic),
            )
        })
    }
}
