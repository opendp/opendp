use crate::core::{Domain, Measure, Measurement, Metric, MetricSpace};
use std::any::{type_name, Any};
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::*;

/// A queryable is like a state machine:
/// 1. it takes an input of type `Query<Q>`,
/// 2. updates its internal state,
/// 3. and emits an answer of type `Answer<A>`
pub struct Queryable<Q: ?Sized, A>(Rc<RefCell<dyn FnMut(&Self, Query<Q>) -> Fallible<Answer<A>>>>);

impl<Q: ?Sized, A> Queryable<Q, A> {
    pub fn eval(&mut self, query: &Q) -> Fallible<A> {
        match self.eval_query(Query::External(query))? {
            Answer::External(ext) => Ok(ext),
            Answer::Internal(_) => fallible!(
                FailedFunction,
                "cannot return internal answer from an external query"
            ),
        }
    }

    pub fn eval_wrap(
        &mut self,
        query: &Q,
        wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    ) -> Fallible<A> {
        wrap(wrapper, || self.eval(query))
    }

    pub(crate) fn eval_internal<'a, AI: 'static>(&mut self, query: &'a dyn Any) -> Fallible<AI> {
        match self.eval_query(Query::Internal(query))? {
            Answer::Internal(value) => value.downcast::<AI>().map(|v| *v).map_err(|_| {
                err!(
                    FailedCast,
                    "could not downcast answer to {}",
                    type_name::<AI>()
                )
            }),
            Answer::External(_) => fallible!(
                FailedFunction,
                "cannot return external answer from an internal query"
            ),
        }
    }

    #[inline]
    pub(crate) fn eval_query(&mut self, query: Query<Q>) -> Fallible<Answer<A>> {
        return (self.0.as_ref().borrow_mut())(self, query);
    }
}

// in the Queryable struct definition, this 'a lifetime is supplied by an HRTB after `dyn`, and then elided
#[derive(Debug)]
pub(crate) enum Query<'a, Q: ?Sized> {
    External(&'a Q),
    Internal(&'a dyn Any),
}

pub(crate) enum Answer<A> {
    External(A),
    Internal(Box<dyn Any>),
}

impl<A> Answer<A> {
    pub fn internal<T: 'static>(value: T) -> Self {
        Self::Internal(Box::new(value))
    }
}

thread_local! {
    static WRAPPER: RefCell<Option<Rc<dyn Fn(PolyQueryable) -> Fallible<PolyQueryable>>>> = RefCell::new(None);
}

pub(crate) fn wrap<T>(
    wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    f: impl FnOnce() -> T,
) -> T {
    let prev_wrapper = WRAPPER.with(|w| w.borrow_mut().take());

    let new_wrapper = Some(if let Some(prev) = prev_wrapper.clone() {
        Rc::new(move |qbl| (wrapper)((prev)(qbl)?)) as Rc<_>
    } else {
        Rc::new(wrapper) as Rc<_>
    });

    WRAPPER.with(|w| *w.borrow_mut() = new_wrapper);
    let res = f();
    WRAPPER.with(|w| *w.borrow_mut() = prev_wrapper);
    res
}

impl<DI: Domain, TO, MI: Metric, MO: Measure> Measurement<DI, TO, MI, MO>
where
    (DI, MI): MetricSpace,
{
    pub fn invoke_wrap(
        &self,
        arg: &DI::Carrier,
        wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    ) -> Fallible<TO> {
        wrap(wrapper, || self.invoke(arg))
    }
}

/// WrapFn is a utility for constructing a closure that wraps a PolyQueryable,
/// in a way that recursively wraps any PolyQueryables that are returned.
///
/// The use of a struct avoids an infinite recursion in the type system,
/// as the first argument to the closure is the same type as the closure itself.
#[derive(Clone)]
pub(crate) struct WrapFn(pub Rc<dyn Fn(WrapFn, PolyQueryable) -> Fallible<PolyQueryable>>);
impl WrapFn {
    // constructs a closure that wraps a PolyQueryable
    pub(crate) fn as_map(&self) -> impl Fn(PolyQueryable) -> Fallible<PolyQueryable> {
        let wrap_logic = self.clone();
        move |qbl| (wrap_logic.0)(wrap_logic.clone(), qbl)
    }

    pub(crate) fn new(
        logic: impl Fn(WrapFn, PolyQueryable) -> Fallible<PolyQueryable> + 'static,
    ) -> Self {
        WrapFn(Rc::new(logic))
    }

    pub(crate) fn new_pre_hook(pre_hook: impl FnMut() -> Fallible<()> + 'static) -> Self {
        let pre_hook = Rc::new(RefCell::new(pre_hook));
        WrapFn::new(move |wrap_logic, mut inner_qbl| {
            let pre_hook = pre_hook.clone();
            Ok(Queryable::new_raw(
                move |_wrapper_qbl, query: Query<dyn Any>| {
                    // check the pre_hook for permission to execute
                    (pre_hook.as_ref().borrow_mut())()?;

                    // evaluate the query and wrap the answer
                    wrap(wrap_logic.as_map(), || inner_qbl.eval_query(query))
                },
            ))
        })
    }
}

impl<Q: ?Sized, A> Queryable<Q, A>
where
    Self: IntoPolyQueryable + FromPolyQueryable,
{
    pub(crate) fn new(
        transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<A>> + 'static,
    ) -> Fallible<Self> {
        let queryable = Queryable::new_raw(transition);
        let wrapper = WRAPPER.with(|w| w.borrow().clone());
        Ok(match wrapper {
            None => queryable,
            Some(w) => Queryable::from_poly(w(queryable.into_poly())?),
        })
    }

    pub(crate) fn new_raw(
        transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<A>> + 'static,
    ) -> Self {
        Queryable(Rc::new(RefCell::new(transition)))
    }

    #[allow(dead_code)]
    pub(crate) fn new_external(
        mut transition: impl FnMut(&Q) -> Fallible<A> + 'static,
    ) -> Fallible<Self> {
        Queryable::new(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<A>> {
                match query {
                    Query::External(q) => transition(q).map(Answer::External),
                    Query::Internal(_) => fallible!(FailedFunction, "unrecognized internal query"),
                }
            },
        )
    }

    pub(crate) fn new_raw_external(
        mut transition: impl FnMut(&Q) -> Fallible<A> + 'static,
    ) -> Self {
        Queryable::new_raw(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<A>> {
                match query {
                    Query::External(q) => transition(q).map(Answer::External),
                    Query::Internal(_) => fallible!(FailedFunction, "unrecognized internal query"),
                }
            },
        )
    }
}

// manually implemented instead of derived so that Q and A don't have to be Clone
impl<Q: ?Sized, A> Clone for Queryable<Q, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub type PolyQueryable = Queryable<dyn Any, Box<dyn Any>>;

pub trait IntoPolyQueryable {
    fn into_poly(self) -> PolyQueryable;
}

impl<Q: 'static, A: 'static> IntoPolyQueryable for Queryable<Q, A> {
    fn into_poly(mut self) -> PolyQueryable {
        Queryable::new_raw(move |_self: &PolyQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q) => {
                    let answer = self.eval(q.downcast_ref::<Q>().ok_or_else(|| {
                        err!(FailedCast, "query must be of type {}", type_name::<Q>())
                    })?)?;
                    Answer::External(Box::new(answer))
                }
                Query::Internal(q) => {
                    let Answer::Internal(a) = self.eval_query(Query::Internal(q))? else {
                        return fallible!(
                            FailedFunction,
                            "internal query returned external answer"
                        );
                    };
                    Answer::Internal(a)
                }
            })
        })
    }
}

// The previous impl over all Q has an implicit `Sized` trait bound, whereas this parameterizes Q as dyn Any, which is not Sized.
// Therefore, the compiler recognizes these impls as disjoint.
impl IntoPolyQueryable for PolyQueryable {
    fn into_poly(self) -> PolyQueryable {
        // if already a PolyQueryable, no need to do anything.
        self
    }
}

pub trait FromPolyQueryable {
    fn from_poly(v: PolyQueryable) -> Self;
}

impl<Q: 'static, A: 'static> FromPolyQueryable for Queryable<Q, A> {
    fn from_poly(mut self_: PolyQueryable) -> Self {
        Queryable::new_raw(move |_self: &Queryable<Q, A>, query: Query<Q>| {
            Ok(match query {
                Query::External(query) => {
                    let answer = self_.eval(query)?;

                    let answer = *answer.downcast::<A>().map_err(|_| {
                        err!(FailedCast, "failed to downcast to {:?}", type_name::<A>())
                    })?;
                    Answer::External(answer)
                }
                Query::Internal(q) => {
                    let Answer::Internal(a) = self_.eval_query(Query::Internal(q))? else {
                        return fallible!(
                            FailedFunction,
                            "internal query returned external answer"
                        );
                    };
                    Answer::Internal(a)
                }
            })
        })
    }
}

// The previous impl over all Q has an implicit `Sized` trait bound, whereas this parameterizes Q as dyn Any, which is not Sized.
// Therefore, the compiler recognizes these impls as disjoint.
impl FromPolyQueryable for PolyQueryable {
    fn from_poly(self_: Self) -> Self {
        // if already a PolyQueryable, no need to do anything.
        self_
    }
}

impl<Q: ?Sized> Queryable<Q, Box<dyn Any>> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(query)?
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
