use crate::error::*;
use std::any::{Any, type_name};
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// A queryable is like a state machine:
/// 1. it takes an input of type `Query<Q>`,
/// 2. updates its internal state,
/// 3. and emits an answer of type `Answer<A>`
pub struct Queryable<Q: ?Sized, A>(
    Arc<Mutex<dyn FnMut(&Self, Query<Q>) -> Fallible<Answer<A>> + Send + Sync>>,
);

impl<Q: ?Sized, A> Queryable<Q, A> {
    pub fn eval(&mut self, query: &Q) -> Fallible<A> {
        match self.eval_query(Query::External(query, None))? {
            Answer::External(ext) => Ok(ext),
            Answer::Internal(_) => fallible!(
                FailedFunction,
                "cannot return internal answer from an external query"
            ),
        }
    }

    pub fn eval_wrap(&mut self, query: &Q, wrapper: Option<Wrapper>) -> Fallible<A> {
        match self.eval_query(Query::External(query, wrapper))? {
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
        self.0.as_ref().lock()?(self, query)
    }
}

// in the Queryable struct definition, this 'a lifetime is supplied by an HRTB after `dyn`, and then elided
pub(crate) enum Query<'a, Q: ?Sized> {
    External(&'a Q, Option<Wrapper>),
    Internal(&'a dyn Any),
}

impl<'a, Q: ?Sized + std::fmt::Debug> std::fmt::Debug for Query<'a, Q> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External(arg0, _) => f.debug_tuple("External").field(arg0).finish(),
            Self::Internal(arg0) => f.debug_tuple("Internal").field(arg0).finish(),
        }
    }
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

pub(crate) fn compose_wrappers(outer: Option<Wrapper>, inner: Option<Wrapper>) -> Option<Wrapper> {
    option_or_merge(inner, outer, |outer, inner| {
        Wrapper::new(move |queryable| outer(inner(queryable)?))
    })
}

fn option_or_merge<T>(outer: Option<T>, inner: Option<T>, merge: impl Fn(T, T) -> T) -> Option<T> {
    match (outer, inner) {
        (Some(l), Some(r)) => Some(merge(l, r)),
        (l, r) => l.or(r),
    }
}

/// Wrapper is a function that wraps a Queryable in another Queryable.
#[derive(Clone)]
pub struct Wrapper(Arc<dyn Fn(PolyQueryable) -> Fallible<PolyQueryable> + Send + Sync + 'static>);

// make Wrapper callable as a function
impl Deref for Wrapper {
    type Target = dyn Fn(PolyQueryable) -> Fallible<PolyQueryable>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Wrapper {
    pub fn new(
        wrapper: impl Fn(PolyQueryable) -> Fallible<PolyQueryable> + Send + Sync + 'static,
    ) -> Self {
        Wrapper(Arc::new(wrapper))
    }

    /// Creates a recursive wrapper that recursively applies itself to child queryables.
    /// `hook` is called any time the wrapped queryable or any of its children are queried.
    pub fn new_recursive_pre_hook(
        hook: impl FnMut() -> Fallible<()> + Send + Sync + Clone + 'static,
    ) -> Wrapper {
        RecursiveWrapper(Arc::new(move |recursive_wrapper, mut inner_qbl| {
            let mut hook = hook.clone();
            Ok(Queryable::new(move |_, mut query: Query<dyn Any>| {
                // call the hook
                hook()?;

                if let Query::External(_, query_wrapper) = &mut query {
                    *query_wrapper = compose_wrappers(
                        Some(recursive_wrapper.clone().to_wrapper()),
                        query_wrapper.clone(),
                    );
                }
                // evaluate the query and wrap the answer
                inner_qbl.eval_query(query)
            }))
        }))
        .to_wrapper()
    }
}

/// RecursiveWrapper is a utility for constructing a closure that wraps a Queryable,
/// in a way that recursively wraps any children of the Queryable.
// The use of a struct avoids an infinite recursion in the type system,
// as the first argument to the closure is the same type as the closure itself.
#[derive(Clone)]
struct RecursiveWrapper(
    pub Arc<dyn Fn(RecursiveWrapper, PolyQueryable) -> Fallible<PolyQueryable> + Send + Sync>,
);

impl RecursiveWrapper {
    fn to_wrapper(self) -> Wrapper {
        Wrapper::new(move |qbl: PolyQueryable| self.0(self.clone(), qbl))
    }
}

impl<Q: ?Sized, A> Queryable<Q, A>
where
    Self: IntoPolyQueryable + FromPolyQueryable,
{
    pub(crate) fn new(
        transition: impl FnMut(&Self, Query<Q>) -> Fallible<Answer<A>> + 'static + Send + Sync,
    ) -> Self {
        Queryable(Arc::new(Mutex::new(transition)))
    }

    pub(crate) fn wrap(self, wrapper: Option<Wrapper>) -> Fallible<Self> {
        Ok(match wrapper {
            None => self,
            Some(wrap) => Queryable::from_poly(wrap(self.into_poly())?),
        })
    }

    #[allow(dead_code)]
    pub(crate) fn new_external(
        mut transition: impl FnMut(&Q, Option<Wrapper>) -> Fallible<A> + 'static + Send + Sync,
    ) -> Self {
        Queryable::new(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<A>> {
                match query {
                    Query::External(q, wrapper) => transition(q, wrapper).map(Answer::External),
                    Query::Internal(_) => fallible!(FailedFunction, "unrecognized internal query"),
                }
            },
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_raw_external(
        mut transition: impl FnMut(&Q) -> Fallible<A> + 'static + Send + Sync,
    ) -> Self {
        Queryable::new(
            move |_self: &Self, query: Query<Q>| -> Fallible<Answer<A>> {
                match query {
                    Query::External(q, _) => transition(q).map(Answer::External),
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
        Queryable::new(move |_self: &PolyQueryable, query: Query<dyn Any>| {
            Ok(match query {
                Query::External(q, wrapper) => {
                    let q = q.downcast_ref::<Q>().ok_or_else(|| {
                        err!(FailedCast, "query must be of type {}", type_name::<Q>())
                    })?;
                    Answer::External(Box::new(self.eval_wrap(q, wrapper)?))
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
        Queryable::new(move |_self: &Queryable<Q, A>, query: Query<Q>| {
            Ok(match query {
                Query::External(query, wrapper) => {
                    let answer = self_.eval_wrap(query, wrapper)?;

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
