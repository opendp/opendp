use std::any::{type_name, Any};
use std::cell::RefCell;
use std::rc::Rc;

use crate::combinators::Wrap;
use crate::core::{Domain, Measure, Measurement, Metric, Transformation};
use crate::domains::PolyDomain;
use crate::error::*;
use crate::traits::CheckNull;

/// A structure tracking the state of an interactive measurement queryable.

// #[derive(Clone)]
// pub(crate) struct Queryable<Q, QI, A, AI>(
//     // 1. use strong type for query via two generics: for the types of public and private queries
//     //     private queries have a nonce
//     // 2. return is similarly, either public or private. But if public, then either interactive or data
//     Rc<RefCell<dyn FnMut(&Self, &Either<Q, Nonced<QI>>) -> Fallible<Either<Answer<A>, AI>>>>,
// );

// Downside of using a const generic to denote interactivity of output:
//   Can't have compositors that emit both interactive and noninteractive

// pub(crate) struct QueryableOld<Q, QI, DA: Domain, AI>(
//     Rc<RefCell<dyn FnMut(&Self, &Either<Q, Nonced<QI>>) -> Fallible<Either<(DA::Carrier, bool), Box<dyn Any>>>>>,
// );

pub struct Queryable<Q: ?Sized, DA: Domain> (
    Rc<RefCell<dyn FnMut(&Self, Query<Q>) -> Fallible<Answer<DA::Carrier>>>>,
) where DA::Carrier: Sized;

impl<Q: ?Sized, DA: Domain> Clone for Queryable<Q, DA> where DA::Carrier: Sized {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
// without an associated fn on Domain:
// type GeneralQueryable<Q, QI, Q2, A2, AD, AI> = Queryable<Either<Q, Nonced<QI>>, Either<Answer<Queryable<Q2, A2>, AD>, AI>>;
// type BranchQueryable<Q, QI, Q2, A2, AI> = GeneralQueryable<Q, QI, Q2, A2, (), AI>;
// type LeafQueryable<Q, QI, A, AI> = GeneralQueryable<Q, QI, (), (), A, AI>;

// type GeneralUserQueryable<Q, Q2, A2, AD> = Queryable<Q, Answer<Queryable<Q2, A2>, AD>>;
// type BranchUserQueryable<Q, Q2, A2> = GeneralUserQueryable<Q, Q2, A2, ()>;
// type LeafUserQueryable<Q, A> = GeneralUserQueryable<Q, (), (), A>;

pub type PolyQueryable = Queryable<Box<dyn Any>, PolyDomain>;
pub type DynQueryable = Queryable<dyn Any, PolyDomain>;
pub type TempQueryable = DynQueryable;

impl<Q: ?Sized, DA: Domain> Queryable<Q, DA>
    where DA::Carrier: Sized {
    pub fn eval(&mut self, query: &Q) -> Fallible<DA::Carrier> {
        self.eval_meta(query).map(|v| v.0)
    }

    pub(crate) fn eval_meta(&mut self, query: &Q) -> Fallible<(DA::Carrier, bool)> {
        match self.eval_query(Query::External(query))? {
            Answer::External(a, i) => Ok((a, i)),
            Answer::Internal(_) => fallible!(
                FailedFunction,
                "cannot return internal answer from an external query"
            ),
        }
    }

    pub(crate) fn eval_internal<AI: 'static>(&mut self, query: &dyn Any) -> Fallible<AI> {
        match self.eval_query(Query::Internal(query))? {
            Answer::Internal(a) => a.downcast::<AI>().map(|v| *v).map_err(|_| {
                err!(
                    FailedCast,
                    "could not downcast answer to {}",
                    type_name::<AI>()
                )
            }),
            Answer::External(_, _) => fallible!(
                FailedFunction,
                "cannot return external answer from an internal query"
            ),
        }
    }

    pub(crate) fn eval_query(&mut self, query: Query<Q>) -> Fallible<Answer<DA::Carrier>> {
        return (self.0.as_ref().borrow_mut())(self, query);
    }
}

pub(crate) enum Query<'a, Q: ?Sized> {
    External(&'a Q),
    Internal(&'a dyn Any),
}

pub(crate) enum Answer<A> {
    External(A, bool),
    Internal(Box<dyn Any>),
}

impl<A> From<(A, bool)> for Answer<A> {
    fn from((answer, interactive): (A, bool)) -> Self {
        Answer::External(answer, interactive)
    }
}

impl<Q: ?Sized, DA: Domain> Queryable<Q, DA> where DA::Carrier: Sized {
    pub(crate) fn new(
        mut transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<DA::Carrier>> + 'static,
    ) -> Self {
        Queryable(Rc::new(RefCell::new(move |self_: &Self, query: Query<Q>| {
            if let Query::Internal(q) = &query {
                if let Some(wrap) = q.downcast_ref::<Wrap>() {
                    let answer = wrap.0.take().unwrap().downcast::<DA::Carrier>().unwrap();
                    return Ok(Answer::Internal(DA::map_queryable(answer, &*wrap.1)? as Box<dyn Any>))
                }
            }

            transition(self_, query)
        })))
    }

    pub fn new_external(
        mut transition: impl FnMut(&Q) -> Fallible<(DA::Carrier, bool)> + 'static,
    ) -> Self {
        Queryable::new(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<DA::Carrier>> {
                match query {
                    Query::External(q) => transition(q).map(From::from),
                    Query::Internal(_) => fallible!(FailedFunction, "unrecognized internal query"),
                }
            },
        )
    }
}


impl<DA: Domain> Queryable<(), DA> where DA::Carrier: Sized {
    pub fn get(&mut self) -> Fallible<DA::Carrier> {
        self.eval(&())
    }
}

pub trait IntoDyn {
    fn into_dyn(self) -> DynQueryable;
}

impl<DA: Domain> IntoDyn for Queryable<dyn Any, DA> where DA::Carrier: Sized {
    fn into_dyn(mut self) -> DynQueryable {
        Queryable::new(move |_self: &DynQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q) => {
                    let (answer, interactive) = self.eval_meta(q)?;
                    Answer::External(Box::new(answer) as Box<dyn Any>, interactive)
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
            })
        })
    }
}

impl<Q: Sized, DA: Domain> IntoDyn for Queryable<Q, DA> where DA::Carrier: Sized {
    fn into_dyn(self) -> DynQueryable {
        Queryable::new(move |_self: &DynQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q) => {
                    let (answer, interactive) =
                        self.eval_meta(q.downcast_ref::<Q>().ok_or_else(|| {
                            err!(FailedCast, "query must be of type {}", type_name::<Q>())
                        })?)?;
                    Answer::External(Box::new(answer) as Box<dyn Any>, interactive)
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
            })
        })
    }
    
}


impl<Q: 'static, DA: Domain> Queryable<Q, DA> where DA::Carrier: Sized {
    pub fn into_poly(mut self) -> PolyQueryable {
        Queryable::new(move |_self: &PolyQueryable, query: Query<Box<dyn Any>>| {
            Ok(match query {
                Query::External(q) => {
                    let (answer, interactive) =
                        self.eval_meta(q.downcast_ref::<Q>().ok_or_else(|| {
                            err!(FailedCast, "query must be of type {}", type_name::<Q>())
                        })?)?;
                    Answer::External(Box::new(answer) as Box<dyn Any>, interactive)
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
            })
        })
    }
}

pub trait FromDyn {
    fn from_dyn(v: DynQueryable) -> Self;
}

impl<Q: 'static, DA: Domain> FromDyn for Queryable<Q, DA> where DA::Carrier: Sized {
    fn from_dyn(self_: DynQueryable) -> Self {
        Queryable::new(move |_self: &Queryable<Q, DA>, query: Query<Q>| {
            Ok(match query {
                Query::External(query) => {
                    let (answer, interactive) = self_.eval_meta(query as &dyn Any)?;

                    let answer = *answer.downcast::<DA::Carrier>()
                        .map_err(|_| err!(FailedCast, "failed to downcast to {:?}", type_name::<DA::Carrier>()))?;
                    Answer::External(answer, interactive)
                }
                Query::Internal(q) => Answer::Internal(self_.eval_internal(q)?),
            })
        })
    }
}

impl<DA: Domain> FromDyn for Queryable<dyn Any, DA> where DA::Carrier: Sized {
    fn from_dyn(self_: DynQueryable) -> Self {
        Queryable::new(move |_self: &Queryable<dyn Any, DA>, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(query) => {
                    let (answer, interactive) = self_.eval_meta(query)?;

                    let answer = *answer.downcast::<DA::Carrier>()
                        .map_err(|_| err!(FailedCast, "failed to downcast to {:?}", type_name::<DA::Carrier>()))?;
                    Answer::External(answer, interactive)
                }
                Query::Internal(q) => Answer::Internal(self_.eval_internal(q)?),
            })
        })
    }
}

pub trait DowncastDyn<Q: ?Sized, DA: Domain> where DA::Carrier: Sized {
    fn downcast_dyn(self) -> Queryable<Q, DA>;
}

impl<Q: ?Sized, DA: Domain> DowncastDyn<Q, DA> for DynQueryable 
    where DA::Carrier: Sized, Queryable<Q, DA>: FromDyn {
    fn downcast_dyn(self) -> Queryable<Q, DA> {
        Queryable::<Q, DA>::from_dyn(self)
    }
}



impl<Q: 'static, DA: Domain> From<Queryable<dyn Any, PolyDomain>> for Queryable<Q, DA>
    where DA::Carrier: Sized {
    fn from(self_: Queryable<dyn Any, PolyDomain>) -> Self {
        Queryable::new(move |_self: &Queryable<Q, DA>, query: Query<Q>| {
            Ok(match query {
                Query::External(query) => {
                    let (answer, interactive) = self_.eval_meta(query as &dyn Any)?;

                    let answer = *answer.downcast::<DA::Carrier>()
                        .map_err(|_| err!(FailedCast, "failed to downcast to {:?}", type_name::<DA::Carrier>()))?;
                    Answer::External(answer, interactive)
                }
                Query::Internal(q) => Answer::Internal(self_.eval_internal(q)?),
            })
        })
    }
}


impl PolyQueryable {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<Q: 'static, A: 'static>(&mut self, query: Q) -> Fallible<A> {
        self.eval(&(Box::new(query) as Box<dyn Any>))?
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

    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn get_poly<A: 'static>(&mut self) -> Fallible<A> {
        self.eval_poly(&())
    }
}

impl Queryable<dyn Any, PolyDomain> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn get_dyn<A: 'static>(&mut self) -> Fallible<A> {
        self.eval(&())?
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

impl<Q: 'static + ?Sized> Queryable<Q, PolyDomain> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_dyn<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
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

impl<Q, DA: Domain> CheckNull for Queryable<Q, DA> where DA::Carrier: Sized {
    fn is_null(&self) -> bool {
        false
    }
}

impl<DI: Domain, DOQ: Domain, DOA: Domain, MI: Metric, MO: Measure> CheckNull
    for Measurement<DI, DOQ, DOA, MI, MO> 
    where DOA::Carrier: Sized,
          Queryable<DOQ::Carrier, DOA>: IntoDyn + FromDyn
{
    fn is_null(&self) -> bool {
        false
    }
}

impl<DI: Domain, DO: Domain, MI: Metric, MO: Metric> CheckNull for Transformation<DI, DO, MI, MO> 
    where DO::Carrier: Sized {
    fn is_null(&self) -> bool {
        false
    }
}
