use std::any::Any;
use std::rc::Rc;

use crate::error::*;
use crate::traits::CheckNull;


/// A structure tracking the state of an interactive measurement queryable.
/// It's generic over state (S), query (Q), answer (A), so it can be used for any
/// interactive measurement expressible as a transition function.
pub struct Queryable<S, Q, A> {
    /// The state of the Queryable. It is wrapped in an option so that ownership can be moved out
    /// temporarily, during transitions.
    state: Option<S>,
    /// The transition function of the Queryable. Takes the current state and a query, returns
    /// the new state and the answer.
    transition: Rc<dyn Fn(S, &dyn Query<Q>) -> Fallible<(S, A)>>,
}

pub trait Query<Q> {
    fn as_Q(&self) -> &Q {
        self
    }
}

impl<S, Q, A> Queryable<S, Q, A> {
    /// Constructs a Queryable with initial state and transition function.
    pub fn new(initial: S, transition: impl Fn(S, &Q) -> Fallible<(S, A)> + 'static) -> Self {
        Queryable {
            state: Some(initial),
            transition: Rc::new(transition),
        }
    }

    /// Evaluates a query.
    pub fn eval(&mut self, query: &dyn Query<Q>) -> Fallible<A> {
        // Take temporary ownership of the state from this struct.
        let state = self
            .state
            .take()
            .unwrap_assert("Queryable state is only accessed in this method, always replaced.");
        // Obtain then new state and answer.
        let new_state_answer = (self.transition)(state, query)?;
        // Restore ownership of the state into this struct.
        self.state.replace(new_state_answer.0);
        Ok(new_state_answer.1)
    }
}

impl<S, Q> Queryable<S, Q, Box<dyn Any>> {
    /// Evaluates a polymorphic query and downcasts to the given type.
    pub fn eval_poly<A: 'static>(&mut self, query: &Q) -> Fallible<A> {
        self.eval(query)?
            .downcast()
            .map_err(|_| err!(FailedCast))
            .map(|b| *b)
    }
}

impl<S, Q, A> CheckNull for Queryable<S, Q, A> {
    fn is_null(&self) -> bool {
        false
    }
}
