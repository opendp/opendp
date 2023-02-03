use std::any::{type_name, Any};
use std::cell::RefCell;
use std::rc::Rc;

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

pub type PolyQueryable = Queryable<dyn Any, PolyDomain>;

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
        transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<DA::Carrier>> + 'static,
    ) -> Self {
        Queryable(Rc::new(RefCell::new(transition)))
        // Queryable(Rc::new(RefCell::new(move |_self: &Self, query: Query<Q>| {
        //     if let Query::Internal(q) = &query {
        //         if let Some(wrap) = q.downcast_ref::<Wrap>() {
        //             return Ok(Answer::Internal(Box::new()))
        //         }
        //     }
        //     match query {
        //         Vis::External(_) => todo!(),
        //         Vis::Internal(_) => todo!(),
        //     }
        // })))
    }

    // pub(crate) fn new_internal<QI: 'static, AI: 'static>(
    //     mut transition: impl FnMut(&Self, Vis<Q, (&QI, Nonce)>) -> Fallible<Vis<(DA::Carrier, bool), AI>>
    //         + 'static,
    // ) -> Self {
    //     Queryable::new(
    //         move |self_: &Self,
    //               query: Vis<Q, (&dyn Any, Nonce)>|
    //               -> Fallible<Vis<(DA::Carrier, bool), Box<dyn Any>>> {
    //             match query {
    //                 Vis::External(query) => {
    //                     let answer = transition(self_, Vis::External(query))?;

    //                     if let Vis::External(answer) = answer {
    //                         Ok(Vis::External(answer))
    //                     } else {
    //                         fallible!(
    //                             FailedFunction,
    //                             "external queries cannot return internal answers"
    //                         )
    //                     }
    //                 }
    //                 Vis::Internal((q, nonce)) => {
    //                     // downcast and upcast
    //                     let q: &QI = q.downcast_ref().ok_or_else(|| {
    //                         err!(FailedCast, "could not downcast to {}", type_name::<Q>())
    //                     })?;
    //                     let answer = transition(self_, Vis::Internal((q, nonce)))?;

    //                     if let Vis::Internal(answer) = answer {
    //                         Ok(Vis::Internal(Box::new(answer) as Box<dyn Any>))
    //                     } else {
    //                         fallible!(
    //                             FailedFunction,
    //                             "internal queries cannot return external answers"
    //                         )
    //                     }
    //                 }
    //             }
    //         },
    //     )
    // }

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

impl<Q: 'static, DA: Domain> Queryable<Q, DA> where DA::Carrier: Sized {
    pub fn to_poly(mut self) -> PolyQueryable {
        Queryable::new(move |_self: &PolyQueryable, query: Query<dyn Any>| {
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


impl<DA: Domain> Queryable<dyn Any, DA> where DA::Carrier: Sized {
    pub fn to_poly(mut self) -> PolyQueryable {
        Queryable::new(move |_self: &PolyQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q) => {
                    let (answer, interactive) =
                        self.eval_meta(q)?;
                    Answer::External(Box::new(answer) as Box<dyn Any>, interactive)
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
            })
        })
    }
}

impl Queryable<dyn Any, PolyDomain> {
    pub fn downcast_qbl<Q: 'static, DA: Domain>(mut self) -> Queryable<Q, DA> where DA::Carrier: Sized {
        Queryable::new(move |_self: &Queryable<Q, DA>, query: Query<Q>| {
            Ok(match query {
                Query::External(query) => {
                    let (answer, interactive) = self.eval_meta(query as &dyn Any)?;

                    let answer = *answer.downcast::<DA::Carrier>().unwrap();
                    Answer::External(answer, interactive)
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
            })
            // self.eval(&(Box::new(query.clone()) as Box<dyn Any>))?.downcast()
            // .map_err(|_| {
            //     err!(
            //         FailedCast,
            //         "Failed downcast of eval_poly result to {}",
            //         any::type_name::<DA::Carrier>()
            //     )
            // })
            // .map(|res| *res)
        })
    }
}

impl<DA: Domain> Queryable<(), DA> where DA::Carrier: Sized {
    pub fn get(&mut self) -> Fallible<DA::Carrier> {
        self.eval(&())
    }
}

impl Queryable<dyn Any, PolyDomain> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn get_poly<A: 'static>(&mut self) -> Fallible<A> {
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

impl<Q, DA: Domain> CheckNull for Queryable<Q, DA> where DA::Carrier: Sized {
    fn is_null(&self) -> bool {
        false
    }
}

impl<DI: Domain, DOQ: Domain, DOA: Domain, MI: Metric, MO: Measure> CheckNull
    for Measurement<DI, DOQ, DOA, MI, MO> 
    where DOA::Carrier: Sized
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
