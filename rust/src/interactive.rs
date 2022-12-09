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
}

/// A structure tracking the state of an interactive measurement queryable.
#[derive(Clone)]
pub(crate) struct QueryableBase(
    Rc<RefCell<Option<Box<dyn FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>>>>>>,
);

impl QueryableBase {
    pub fn eval<Q: 'static, A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval_any(query as &dyn Any)?
            .downcast::<A>()
            .map_err(|_| err!(FailedFunction, "failed to downcast"))
            .map(|x| *x)
    }

    pub fn eval_any(&mut self, query: &dyn Any) -> Fallible<Box<dyn Any>> {
        (self.0.borrow_mut().as_mut().unwrap())(self, query)
    }

    pub fn new(
        transition: impl FnMut(&Self, &dyn Any) -> Fallible<Box<dyn Any>> + 'static,
    ) -> Self {
        QueryableBase(Rc::new(RefCell::new(Some(Box::new(transition)))))
    }
}

/// Queryables are used to model interactive measurements,
/// and are generic over the type of the query (Q) and answer (A).
pub struct Queryable<Q, DA: Domain> {
    _query: PhantomData<Q>,
    _answer: PhantomData<DA>,
    pub(crate) base: QueryableBase,
}

impl<Q, DA> Queryable<Q, DA>
where
    Q: 'static + Clone, DA: Domain + 'static, DA::Carrier: 'static
{
    pub fn eval(&mut self, q: &Q) -> Fallible<DA::Carrier> {
        self.base.eval::<Q, DA::Carrier>(q)
    }

    pub fn eval_privacy_after<QD: 'static>(&mut self, q: &Q) -> Fallible<QD> {
        self.base.eval(&PrivacyUsageAfter(q.clone()))
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

    pub(crate) fn new_concrete(
        mut transition: impl FnMut(&Q) -> Fallible<DA::Carrier> + 'static,
    ) -> Self {
        Queryable::new(move |_self: &QueryableBase, query: &dyn Any| {
            let concrete_query = query
                .downcast_ref::<Q>()
                .ok_or_else(|| err!(FailedFunction, "unrecognized query. Expected {}", std::any::type_name::<Q>()))?;
            Ok(Box::new(transition(concrete_query)?))
        })
    }
}

/// Mutates a queryable so that it knows how to speak with its parent
/// Intercepts two kinds of queries:
/// 1. user queries to self
/// 2. internal change queries from children
pub(crate) fn inject_context<Q, DA, QD>(
    queryable: &mut Queryable<Q, DA>,
    mut context: Context,
    d_final: Option<QD>
)
where
    Q: 'static + Clone,
    DA: Domain,
    QD: 'static + Clone,
{
    queryable
        .base
        .0
        .as_ref()
        .replace_with(|f| {
            let mut transition = f.take().unwrap();    
            Some(Box::new(move |self_: &QueryableBase, query: &dyn Any| {
                if let Some(query_typed) = query.downcast_ref::<Q>() {

                    let Some(d_mid) = d_final.clone() else {
                        *transition(self_, &PrivacyUsageAfter(query_typed.clone()))?.downcast().unwrap()
                    };
        
                    context.parent.eval(&ChildChange {
                        id: context.id,
                        new_privacy_loss: d_mid.clone(),
                        commit: false,
                    })?;
        
                    let answer = transition(self_, query)?;
        
                    context.parent.eval(&ChildChange {
                        id: context.id,
                        new_privacy_loss: d_mid.clone(),
                        commit: true,
                    })?;
        
                    return Ok(Box::new(answer));
                }
                // 1. sequential composition
                // 2. concurrent composition
                // 3. sparse vector
                if let Some(query) = query.downcast_ref::<ChildChange<QD>>() {
                    // checks with the inner queryable if the child change is ok
                    // TODO: don't unwrap
                    let d_mid: QD = *transition(self_, query)?.downcast().unwrap();
                    
                    // check with the parent that the change to the inner queryable is ok
                    context.parent.eval_any(&ChildChange {
                        id: context.id,
                        new_privacy_loss: d_mid.clone(),
                        commit: query.commit,
                    })?;

                    return Ok(Box::new(d_mid))
                }
        
                transition(self_, query)
            }))
        });
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
    pub id: usize,
    pub new_privacy_loss: Q,
    pub commit: bool,
}

pub(crate) struct PrivacyUsage;
pub(crate) struct PrivacyUsageAfter<Q>(pub Q);

impl<Q, DA: Domain> CheckNull for Queryable<Q, DA> {
    fn is_null(&self) -> bool {
        false
    }
}
