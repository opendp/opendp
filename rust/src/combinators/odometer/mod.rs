pub mod sequential;

pub mod filter;
pub use filter::*;

use std::any::Any;

use crate::{
    core::{Measure, Measurement, Metric},
    error::Fallible,
    interactive::Queryable,
};

pub type OdometerQueryable<QI, QM, AI, AM> =
    Queryable<OdometerQuery<QI, QM>, OdometerAnswer<AI, AM>>;

pub type OdometerCompositor<DI, TO, MI, MO> = OdometerQueryable<
    Measurement<DI, TO, MI, MO>,
    <MI as Metric>::Distance,
    TO,
    <MO as Measure>::Distance,
>;

pub enum OdometerQuery<QI, QM> {
    Invoke(QI),
    Map(QM),
}

pub enum OdometerAnswer<AI, AM> {
    Invoke(AI),
    Map(AM),
}

// convenience methods for invoking or mapping distances over the odometer queryable
impl<QI, QM: 'static, AI, AM: 'static> Queryable<OdometerQuery<QI, QM>, OdometerAnswer<AI, AM>> {
    pub fn eval_invoke(&mut self, query: QI) -> Fallible<AI> {
        if let OdometerAnswer::Invoke(answer) = self.eval(&OdometerQuery::Invoke(query))? {
            Ok(answer)
        } else {
            fallible!(FailedCast, "return type is not an answer")
        }
    }
    pub fn eval_map(&mut self, d_in: QM) -> Fallible<AM> {
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
                    "eval_invoke_poly failed to downcast to {}",
                    std::any::type_name::<A>()
                )
            })
            .map(|b| *b)
    }
}
