use std::any::Any;

pub mod basic;
pub mod filter;

use num::Zero;

use crate::{
    core::{Domain, Measure, Measurement, Metric, Odometer, PrivacyMap},
    error::Fallible,
    interactive::{Answer, PolyQueryable, Query, Queryable, QueryableMap},
};

pub type OdometerQueryable<Q, A, QU, AU> = Queryable<OdometerQuery<Q, QU>, OdometerAnswer<A, AU>>;

pub enum OdometerQuery<Q, U> {
    Invoke(Q),
    Map(U),
}

pub enum OdometerAnswer<A: QueryableMap, U> {
    Invoke(A),
    Map(U),
}

// define how to map over queryables in the output
impl<A: QueryableMap, U: 'static> QueryableMap for OdometerAnswer<A, U> {
    fn queryable_map(self, mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        match self {
            OdometerAnswer::Invoke(answer) => OdometerAnswer::Invoke(answer.queryable_map(mapper)),
            answer => answer,
        }
    }
}

// convenience eval's for invoking of mapping distances over the odometer queryable
impl<Q, A: QueryableMap, QU: 'static, AU: 'static>
    Queryable<OdometerQuery<Q, QU>, OdometerAnswer<A, AU>>
{
    pub fn eval_invoke(&mut self, query: Q) -> Fallible<A> {
        if let OdometerAnswer::Invoke(answer) = self.eval(&OdometerQuery::Invoke(query))? {
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

pub trait Invokable<DI: Domain, MI: Metric, MO: Measure> {
    type Output: QueryableMap;
    fn invoke(
        &self,
        value: &DI::Carrier,
        parent: PolyQueryable,
        id: usize,
    ) -> Fallible<Self::Output>;

    fn privacy_map(&self) -> PrivacyMap<MI, MO>;

    fn input_domain(&self) -> DI;
    fn input_metric(&self) -> MI;
    fn output_measure(&self) -> MO;
}

impl<DI: Domain, TO: QueryableMap, MI: Metric, MO: Measure> Invokable<DI, MI, MO>
    for Measurement<DI, TO, MI, MO>
{
    type Output = TO;
    fn invoke(
        &self,
        value: &DI::Carrier,
        _parent: PolyQueryable,
        _id: usize,
    ) -> Fallible<Self::Output> {
        self.invoke(value)
    }
    fn privacy_map(&self) -> PrivacyMap<MI, MO> {
        self.privacy_map.clone()
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

impl<DI: Domain + 'static, TO: QueryableMap, MI: Metric + 'static, MO: Measure + 'static>
    Invokable<DI, MI, MO> for Odometer<DI, TO, MI, MO>
where
    MI::Distance: Clone,
    MO::Distance: Zero,
{
    type Output = TO;
    fn invoke(
        &self,
        value: &DI::Carrier,
        parent: PolyQueryable,
        id: usize,
    ) -> Fallible<Self::Output> {
        // wrap the child odometer to send ChildChange queries
        Ok(self.invoke(value)?.queryable_map(&move |mut inner_qbl| {
            let mut parent = parent.clone();

            Queryable::new(move |_self: &PolyQueryable, query: Query<dyn Any>| {
                match query {
                    Query::External(_) => inner_qbl.eval_query(query),
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
                                id: Some(id),
                                new_privacy_map: pending_map.clone(),
                                commit: change_query.commit,
                            })?;

                            // return the pending map to the caller
                            return Ok(Answer::internal(pending_map));
                        }

                        return fallible!(FailedFunction, "unrecognized internal query");
                    }
                }
            })
        }))
    }

    fn privacy_map(&self) -> PrivacyMap<MI, MO> {
        PrivacyMap::new(|_d_in| MO::Distance::zero())
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
struct ChildChange<MI: Metric, MO: Measure> {
    pub id: Option<usize>,
    pub new_privacy_map: PrivacyMap<MI, MO>,
    pub commit: bool,
}
