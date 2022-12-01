use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::*;
use crate::traits::CheckNull;

#[derive(Clone)]
pub struct Context {
    pub parent: Queryable,
    pub id: usize
}

#[derive(Clone)]
pub struct Node {
    pub value: Rc<dyn Any>,
    pub context: Option<Context>
}

/// A structure tracking the state of an interactive measurement queryable.
/// It's generic over state (S), query (Q), answer (A), so it can be used for any
/// interactive measurement expressible as a transition function.
// pub struct Queryable<S> {
//     /// The state of the Queryable. It is wrapped in an option so that ownership can be moved out
//     /// temporarily, during transitions.
//     pub state: S,
//     /// The transition function of the Queryable. Takes the current state and a query, returns
//     /// the new state and the answer.
//     pub transition: Rc<dyn Fn(&Queryable<S>, &dyn Any) -> Fallible<(S, Box<dyn Any>)>>,
// }

// Queryables don't need separate state:
#[derive(Clone)]
pub struct Queryable(Rc<RefCell<
    dyn FnMut(&Queryable, &dyn Any) -> Fallible<Box<dyn Any>>
>>);

impl Queryable {
    pub fn eval<A: 'static>(&mut self, q: &dyn Any) -> Fallible<A> {
        let boxed = (self.0.borrow_mut())(self, q)?;
        boxed.downcast::<A>().map_err(|_| err!(FailedFunction, "failed to downcast")).map(|x| *x)
    }
}

// impl<'s, Q, A> Queryable<'s, Q, A> {
//     fn into_any(&self) -> AnyQueryable {
//         let Queryable {state, transition} = self;
//         Queryable { 
//             state: state.map(|s| Box::new(s) as Box<dyn Any>), 
//             transition: Rc::new(move |s: Box<dyn Any>, q: &dyn Any| -> Fallible<(Box<dyn Any>, Box<dyn Any>)> {
//                 let q = q.downcast_ref::<Q>().unwrap();
//                 (transition)(s, q).map(|(s, a)| (s, Box::new(a)))
//             })
//         }
//     }
// }

impl Queryable {
    /// Constructs a Queryable with initial state and transition function.
    pub fn new(transition: impl FnMut(&Queryable, &dyn Any) -> Fallible<Box<dyn Any>> + 'static) -> Self {
        Queryable(Rc::new(RefCell::new(transition)))
    }

    // /// Evaluates a query.
    // pub fn eval(&mut self, query: &dyn Any) -> Fallible<Box<dyn Any>> {
    //     // Take temporary ownership of the state from this struct.
    //     // let state = self
    //     //     .state
    //     //     .take()
    //     //     .unwrap_assert("Queryable state is only accessed in this method, always replaced.");

    //     // Obtain then new state and answer.
    //     let (state, answer) = (self.transition)(&self, query)?;
    //     // Restore ownership of the state into this struct.
    //     self.state = state;
    //     Ok(answer)
    // }
}

// impl<'a, Q> Queryable<'a, Q, Box<dyn Any>> {
//     /// Evaluates a polymorphic query and downcasts to the given type.
//     pub fn eval_poly<A: 'static>(&mut self, query: Q) -> Fallible<A> {
//         self.eval(query)?
//             .downcast()
//             .map_err(|_| err!(FailedCast))
//             .map(|b| *b)
//     }
// }

impl CheckNull for Queryable {
    fn is_null(&self) -> bool {
        false
    }
}
