use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::domains::type_name;
use crate::error::*;
use crate::traits::CheckNull;

#[derive(Clone)]
pub struct Context {
    pub parent: QueryableBase,
    pub id: usize,
}

#[derive(Clone)]
pub struct Node {
    pub value: Rc<dyn Any>,
    pub context: Option<Context>,
}

/// A structure tracking the state of an interactive measurement queryable.
#[derive(Clone)]
pub struct QueryableBase(
    Rc<RefCell<dyn FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>>>>
);

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

    pub fn as_typed<Q, A>(self) -> Queryable<Q, A> {
        Queryable {
            _query: PhantomData::<Q>,
            _answer: PhantomData::<A>,
            base: self,
        }
    }
}


/// Queryables are used to model interactive measurements,
/// and are generic over the type of the query (Q) and answer (A).
pub struct Queryable<Q, A> {
    _query: PhantomData<Q>,
    _answer: PhantomData<A>,
    pub base: QueryableBase,
}

impl<Q: 'static, A: 'static> Queryable<Q, A> {
    pub fn eval(&mut self, q: &Q) -> Fallible<A> {
        self.base.eval::<Q, A>(q)
    }

    pub fn new(
        mut transition: impl FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>> + 'static,
    ) -> Self {
        Queryable {
            _query: PhantomData,
            _answer: PhantomData,
            base: QueryableBase::new(move |queryable: &QueryableBase, query: &dyn Any| {
                transition(&queryable.clone().as_typed::<Q, A>(), query)
            }),
        }
    }
}

impl<Q: 'static> Queryable<Q, Box<dyn Any>> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(&query)?
            .downcast()
            .map_err(|_| err!(FailedCast, "failed to downcast to {}", std::any::type_name::<A>()))
            .map(|b| *b)
    }
}


pub struct CheckDescendantChange<Q> {
    pub index: usize,
    pub new_privacy_loss: Q,
    pub commit: bool,
}

// pub trait EvalIfQueryable {
//     // the type that might be evaluated
//     type OutputCarrier;
//     fn eval_if_queryable<Q1: 'static>(value: &mut Self::OutputCarrier, query: Q1) -> Fallible<()>;
// }

// impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> EvalIfQueryable
//     for Measurement<DI, DO, MI, MO>
// {
//     type OutputCarrier = DO::Carrier;
//     fn eval_if_queryable<Q1: 'static>(_value: &mut DO::Carrier, _query: Q1) -> Fallible<()> {
//         Ok(())
//     }
// }

// impl<DI: Domain, Q, A, MI: Metric, MO: Measure> EvalIfQueryable
//     for InteractiveMeasurement<DI, Q, A, MI, MO>
// {
//     type OutputCarrier = QueryableBase;
//     fn eval_if_queryable<Q1: 'static>(queryable: &mut QueryableBase, query: Q1) -> Fallible<()> {
//         queryable.eval(&query)
//     }
// }


impl<Q, A> CheckNull for Queryable<Q, A> {
    fn is_null(&self) -> bool {
        false
    }
}
