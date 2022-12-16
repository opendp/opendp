use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::core::{Domain, Measure, Measurement, Metric, Transformation};
use crate::error::*;
use crate::traits::CheckNull;

/// A structure tracking the state of an interactive measurement queryable.
#[derive(Clone)]
pub(crate) struct QueryableBase(Rc<RefCell<dyn FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>>>>);

impl QueryableBase {
    pub fn eval<Q: 'static, A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval_any(query as &dyn Any)?
            .downcast::<A>()
            .map_err(|_| err!(FailedFunction, "failed to downcast"))
            .map(|x| *x)
    }

    pub fn eval_any(&mut self, query: &dyn Any) -> Fallible<Box<dyn Any>> {
        (self.0.borrow_mut())(self, query)
    }

    pub fn new(
        transition: impl FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>> + 'static,
    ) -> Self {
        QueryableBase(Rc::new(RefCell::new(transition)))
    }
}

/// Queryables are used to model interactive measurements,
/// and are generic over the type of the query (Q) and answer (A).
#[derive(Clone)]
pub struct Queryable<Q: ?Sized, A> {
    _query: PhantomData<Q>,
    _answer: PhantomData<A>,
    pub(crate) base: QueryableBase,
}

impl<Q, A> Queryable<Q, A>
where
    Q: 'static,
    A: 'static,
{
    pub fn eval(&mut self, q: &Q) -> Fallible<A> {
        self.base.eval::<Q, A>(q)
    }

    pub(crate) fn new(
        transition: impl FnMut(&QueryableBase, &dyn Any) -> Fallible<Box<dyn Any>> + 'static,
    ) -> Self {
        Queryable {
            _query: PhantomData,
            _answer: PhantomData,
            base: QueryableBase::new(transition),
        }
    }

    pub(crate) fn new_concrete(mut transition: impl FnMut(&Q) -> Fallible<A> + 'static) -> Self {
        Queryable::new(move |_self: &QueryableBase, query: &dyn Any| {
            let concrete_query = query.downcast_ref::<Q>().ok_or_else(|| {
                err!(
                    FailedFunction,
                    "unrecognized query. Expected {}",
                    std::any::type_name::<Q>()
                )
            })?;
            Ok(Box::new(transition(concrete_query)?))
        })
    }
}

impl<Q, A> Queryable<Q, A>
where
    Q: Clone + 'static,
{
    pub fn eval_privacy_after<QD: 'static>(&mut self, q: &Q) -> Fallible<QD> {
        self.base.eval(&PrivacyUsageAfter(q.clone()))
    }
}

impl<A: 'static> Queryable<(), A> {
    // for consistency with the 1 convention
    pub fn get(&mut self) -> Fallible<A> {
        self.eval(&())
    }
}

impl Queryable<Box<dyn Any>, Box<dyn Any>> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn get_poly<A: 'static>(&mut self) -> Fallible<A> {
        self.eval(&(Box::new(()) as Box<dyn Any>))?
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "failed to downcast to {}",
                    std::any::type_name::<A>()
                )
            })
            .map(|b| *b)
    }
}

impl<Q: 'static> Queryable<Q, Box<dyn Any>> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(query)?
            .downcast()
            .map_err(|_| {
                err!(
                    FailedCast,
                    "failed to downcast to {}",
                    std::any::type_name::<A>()
                )
            })
            .map(|b| *b)
    }
}

pub(crate) struct ChildChange<Q> {
    pub id: usize,
    pub new_privacy_loss: Q,
    pub commit: bool,
}

pub(crate) struct PrivacyUsage;
pub(crate) struct PrivacyUsageAfter<Q>(pub Q);

impl<Q, A> CheckNull for Queryable<Q, A> {
    fn is_null(&self) -> bool {
        false
    }
}

impl<DI: Domain, DOQ: Domain, DOA: Domain, MI: Metric, MO: Measure> CheckNull
    for Measurement<DI, DOQ, DOA, MI, MO>
{
    fn is_null(&self) -> bool {
        false
    }
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> CheckNull for Transformation<DI, DO, MI, MO> {
    fn is_null(&self) -> bool {
        false
    }
}
