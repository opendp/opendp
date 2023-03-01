use std::any::{type_name, Any};
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::*;

/// A queryable is like a state machine:
/// 1. it takes an input of type `Query<Q>`,
/// 2. updates its internal state,
/// 3. and emits an answer of type `Answer<A>`
pub struct Queryable<Q: ?Sized, A: QueryableMap>(
    Rc<RefCell<dyn FnMut(&Self, Query<Q>) -> Fallible<Answer<A>>>>,
);

impl<Q: ?Sized, A: QueryableMap> Queryable<Q, A> {
    pub(crate) fn eval(&mut self, query: &Q) -> Fallible<A::Value> {
        match self.eval_query(Query::External(query))? {
            Answer::External(ext) => Ok(ext.value()),
            Answer::Internal(_) => fallible!(
                FailedFunction,
                "cannot return internal answer from an external query"
            ),
        }
    }

    pub(crate) fn eval_mappable(&mut self, query: &Q) -> Fallible<A> {
        match self.eval_query(Query::External(query))? {
            Answer::External(ext) => Ok(ext),
            Answer::Internal(_) => fallible!(
                FailedFunction,
                "cannot return internal answer from an external query"
            ),
        }
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
pub(crate) enum Query<'a, Q: ?Sized> {
    External(&'a Q),
    Internal(&'a dyn Any),
}

pub(crate) enum Answer<A: QueryableMap> {
    External(A),
    Internal(Box<dyn Any>),
}

impl<A: QueryableMap> Answer<A> {
    pub fn internal<T: 'static>(value: T) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl<Q: ?Sized, A: QueryableMap> Queryable<Q, A> {
    pub(crate) fn new(
        transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<A>> + 'static,
    ) -> Self {
        Queryable(Rc::new(RefCell::new(transition)))
    }

    pub(crate) fn new_external(mut transition: impl FnMut(&Q) -> Fallible<A> + 'static) -> Self {
        Queryable::new(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<A>> {
                match query {
                    Query::External(q) => transition(q).map(Answer::External),
                    Query::Internal(_) => fallible!(FailedFunction, "unrecognized internal query"),
                }
            },
        )
    }
}

impl<A: QueryableMap> Queryable<(), A> {
    pub fn get(&mut self) -> Fallible<A::Value> {
        self.eval(&())
    }
}

// manually implemented instead of derived so that Q and A don't have to be Clone
impl<Q: ?Sized, A: QueryableMap> Clone for Queryable<Q, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub type PolyQueryable = Queryable<dyn Any, Box<dyn QueryableFunctor>>;

pub trait IntoPolyQueryable {
    fn into_poly(self) -> PolyQueryable;
}

impl<Q: 'static, A: QueryableMap> IntoPolyQueryable for Queryable<Q, A> {
    fn into_poly(mut self) -> PolyQueryable {
        Queryable::new(move |_self: &PolyQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q) => {
                    let answer = self.eval_mappable(q.downcast_ref::<Q>().ok_or_else(|| {
                        err!(FailedCast, "query must be of type {}", type_name::<Q>())
                    })?)?;
                    Answer::External(Box::new(answer))
                }
                Query::Internal(q) => Answer::Internal(self.eval_internal(q)?),
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

impl<Q: 'static, A: QueryableMap> FromPolyQueryable for Queryable<Q, A> {
    fn from_poly(mut self_: PolyQueryable) -> Self {
        Queryable::new(move |_self: &Queryable<Q, A>, query: Query<Q>| {
            Ok(match query {
                Query::External(query) => {
                    let answer = self_.eval(query)?;

                    let answer = *answer.into_any().downcast::<A>().map_err(|_| {
                        err!(FailedCast, "failed to downcast to {:?}", type_name::<A>())
                    })?;
                    Answer::External(answer)
                }
                Query::Internal(q) => Answer::Internal(self_.eval_internal(q)?),
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

impl PolyQueryable {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &dyn Any) -> Fallible<A> {
        self.eval(query)?
            .into_any()
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

    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn get_poly<A: 'static>(&mut self) -> Fallible<A> {
        self.eval_poly(&())
    }

    pub fn into_downcast<Q: ?Sized, A: QueryableMap>(self) -> Queryable<Q, A>
    where
        Queryable<Q, A>: FromPolyQueryable,
    {
        Queryable::<Q, A>::from_poly(self)
    }
}

impl<QI: ?Sized> Queryable<QI, PolyQueryable> {
    pub fn eval_poly<Q: 'static + ?Sized, A: QueryableMap>(
        &mut self,
        arg: &QI,
    ) -> Fallible<Queryable<Q, A>>
    where
        Queryable<Q, A>: FromPolyQueryable,
    {
        self.eval(arg).map(Queryable::into_downcast)
    }
}

pub trait QueryableMap: 'static + Sized {
    type Value;
    fn value(self) -> Self::Value;
    fn queryable_map(self, _mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self;
}

impl<Q: 'static + ?Sized, A: QueryableMap> QueryableMap for Queryable<Q, A>
where
    Self: IntoPolyQueryable + FromPolyQueryable,
{
    type Value = Self;
    fn value(self) -> Self::Value {
        self
    }
    fn queryable_map(self, mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        mapper(self.into_poly()).into_downcast()
    }
}
pub struct Static<T>(pub T);
impl<T: 'static> QueryableMap for Static<T> {
    type Value = T;
    fn value(self) -> Self::Value {
        self.0
    }
    fn queryable_map(self, _mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        self
    }
}
pub enum Either<Q, A: QueryableMap, T> {
    Queryable(Queryable<Q, A>),
    Static(T)
}
impl<Q: 'static, A: QueryableMap, T: 'static> QueryableMap for Either<Q, A, T> {
    type Value = Self;
    fn value(self) -> Self {
        self
    }
    fn queryable_map(self, mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        match self {
            Self::Queryable(qbl) => Self::Queryable(qbl.queryable_map(mapper)),
            Self::Static(value) => Self::Static(value)
        }
    }
}

impl<Q: 'static, A: QueryableMap, T: 'static> QueryableMap for (Queryable<Q, A>, Static<T>) {
    type Value = (Queryable<Q, A>, T);
    fn value(self) -> Self::Value {
        (self.0, self.1.0)
    }
    fn queryable_map(self, mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        (self.0.queryable_map(mapper), self.1)
    }
}

impl QueryableMap for Box<dyn QueryableFunctor> {
    type Value = Self;

    fn value(self) -> Self::Value {
        self
    }
    fn queryable_map(self, mapper: &dyn Fn(PolyQueryable) -> PolyQueryable) -> Self {
        self.apply_queryable_map(mapper)
    }
}

// this is the object-safe version of QueryableMap
pub trait QueryableFunctor {
    // makes it possible to downcast from a Box<dyn QueryableFunctor> to T
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn apply_queryable_map(
        self: Box<Self>,
        _mapper: &dyn Fn(PolyQueryable) -> PolyQueryable,
    ) -> Box<dyn QueryableFunctor>;
}

impl<T: QueryableMap + Any> QueryableFunctor for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn apply_queryable_map(
        self: Box<Self>,
        mapper: &dyn Fn(PolyQueryable) -> PolyQueryable,
    ) -> Box<dyn QueryableFunctor> {
        Box::new((*self).queryable_map(mapper))
    }
}