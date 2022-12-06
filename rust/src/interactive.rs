use std::any::Any;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::error::*;
use crate::traits::CheckNull;

#[derive(Clone)]
pub(crate) struct Context {
    pub parent: QueryableBase,
    pub id: usize,
}

impl Context {
    pub fn pre_commit<Q: 'static + Clone>(&mut self, new_privacy_loss: &Q) -> Fallible<()> {
        self.parent.eval(&DescendantChange {
            id: self.id,
            new_privacy_loss: new_privacy_loss.clone(),
            commit: false,
        })
    }

    pub fn commit<Q: 'static + Clone>(&mut self, new_privacy_loss: &Q) -> Fallible<()> {
        self.parent.eval(&DescendantChange {
            id: self.id,
            new_privacy_loss: new_privacy_loss.clone(),
            commit: true,
        })
    }
}

/// A structure tracking the state of an interactive measurement queryable.
#[derive(Clone)]
pub(crate) struct QueryableBase(
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
}


/// Queryables are used to model interactive measurements,
/// and are generic over the type of the query (Q) and answer (A).
pub struct Queryable<Q, A> {
    _query: PhantomData<Q>,
    _answer: PhantomData<A>,
    pub(crate) base: QueryableBase,
}

impl<Q: 'static, A: 'static> Queryable<Q, A> {
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


    pub(crate) fn new_nestable<S: 'static, QD: 'static + Clone>(
        mut state: S,
        pre_eval: impl Fn(&S, &Q) -> Fallible<QD> + 'static,
        eval: impl Fn(&mut S, &Q, &QueryableBase) -> Fallible<A> + 'static,
        pre_child_eval: impl Fn(&S, &DescendantChange<QD>) -> Fallible<QD> + 'static,
        child_eval: impl Fn(&mut S, &DescendantChange<QD>) -> Fallible<()> + 'static
    ) -> Self {
        let mut context: Option<Context> = None;
        Queryable::new(
            move |self_: &QueryableBase, query: &dyn Any| {
                if let Some(query) = query.downcast_ref::<Q>() {

                    let d_mid = pre_eval(&state, &query)?;
                    
                    // if there is context, run a pre-commit
                    if let Some(context) = &mut context {
                        context.pre_commit(&d_mid)?;
                    }

                    let answer = eval(&mut state, &query, self_)?;

                    // if there is context, commit the changes
                    if let Some(context) = &mut context {
                        context.commit(&d_mid)?;
                    }

                    return Ok(Box::new(answer) as Box<dyn Any>);
                }

                if let Some(q) = query.downcast_ref::<Context>() {
                    if context.is_some() {
                        return fallible!(FailedFunction, "context has already been set");
                    }
                    context.replace(q.clone());
                    return Ok(Box::new(()) as Box<dyn Any>);
                }

                // children are always IM's, so new_privacy_loss is bounded by d_mid_i
                if let Some(query) = query.downcast_ref::<DescendantChange<QD>>() {
                    let d_temp = pre_child_eval(&state, query)?;
                    if let Some(context) = &mut context {
                        context.parent.eval_any(&DescendantChange {
                            id: context.id,
                            new_privacy_loss: d_temp,
                            commit: query.commit,
                        })?;
                    };
                    child_eval(&mut state, query)?;
                    return Ok(Box::new(()) as Box<dyn Any>)
                }

                fallible!(FailedFunction, "unrecognized query!")
            }
        )
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


pub(crate) struct DescendantChange<Q> {
    #[allow(dead_code)]
    pub id: usize,
    #[allow(dead_code)]
    pub new_privacy_loss: Q,
    pub commit: bool,
}


impl<Q, A> CheckNull for Queryable<Q, A> {
    fn is_null(&self) -> bool {
        false
    }
}
