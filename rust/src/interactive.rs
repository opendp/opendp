use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::core::Domain;
use crate::domains::PolyDomain;
use crate::error::*;
use crate::traits::CheckNull;

#[derive(Clone)]
pub struct Context {
    parent: QueryableBase,
    id: usize,
}

impl Context {
    pub(crate) fn new(parent: QueryableBase, id: usize) -> Self {
        Context { parent, id }
    }
    pub(crate) fn pre_commit<Q: 'static + Clone>(&mut self, new_privacy_loss: &Q) -> Fallible<()> {
        self.parent.eval(&ChildChange {
            id: self.id,
            new_privacy_loss: new_privacy_loss.clone(),
            commit: false,
        })
    }

    pub(crate) fn commit<Q: 'static + Clone>(&mut self, new_privacy_loss: &Q) -> Fallible<()> {
        self.parent.eval(&ChildChange {
            id: self.id,
            new_privacy_loss: new_privacy_loss.clone(),
            commit: true,
        })
    }
}

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
pub struct Queryable<Q, DA: Domain> {
    _query: PhantomData<Q>,
    _answer: PhantomData<DA>,
    pub(crate) base: QueryableBase,
}

impl<Q: 'static + Clone, DA: Domain + 'static> Queryable<Q, DA>
    where DA::Carrier: 'static {
    
    pub fn eval(&mut self, q: &Q) -> Fallible<DA::Carrier> {
        self.base.eval::<Q, DA::Carrier>(q)
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

    pub(crate) fn with_context<QD: 'static + Clone>(mut self, mut context: Context) -> Self {
        Queryable::new(move |_self_outer: &QueryableBase, query: &dyn Any| {
            if let Some(query) = query.downcast_ref::<Q>() {
                let d_mid = self.base.eval(&PrivacyUsageAfter(query.clone()))?;

                context.pre_commit(&d_mid)?;

                let answer = self.eval(&query)?;

                context.commit(&d_mid)?;

                return Ok(Box::new(answer) as Box<dyn Any>);
            }

            // children are always IM's, so new_privacy_loss is bounded by d_mid_i
            if let Some(query) = query.downcast_ref::<ChildChange<QD>>() {
                let d_temp = self.base.eval(query)?;
                return context.parent.eval_any(&ChildChange {
                    id: context.id,
                    new_privacy_loss: d_temp,
                    commit: query.commit,
                });
            }

            self.base.eval_any(query)
        })
    }
}

impl<Q: 'static + Clone> Queryable<Q, PolyDomain> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(&query)?
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
    #[allow(dead_code)]
    pub id: usize,
    #[allow(dead_code)]
    pub new_privacy_loss: Q,
    pub commit: bool,
}

pub struct PrivacyUsageAfter<Q>(pub Q);

impl<Q, DA: Domain> CheckNull for Queryable<Q, DA> {
    fn is_null(&self) -> bool {
        false
    }
}
