#[allow(unused_variables)]
use std::marker::PhantomData;

pub type Privacy = f64;

pub struct Measurement<I, O> {
    privacy_loss: Privacy,
    // IGNORE THE FOLLOWING, RUST IMPL QUIRK.
    _input_type: PhantomData<I>,
    _output_type: PhantomData<O>,
}
impl<I, O> Measurement<I, O> {
    pub fn new(privacy_loss: Privacy) -> Self {
        Self { privacy_loss, _input_type: PhantomData, _output_type: PhantomData }
    }
    pub fn eval(&self, _input: &I) -> O {
        unimplemented!()
    }
}

pub type Any = Box<dyn std::any::Any>;

pub struct Queryable<P, S, Q, A> {
    parameters: P,
    state: S,
    transition: Box<dyn Fn(&P, &S, &Q) -> (S, A)>,
    // IGNORE THE FOLLOWING, RUST IMPL QUIRK.
    _query_type: PhantomData<Q>,
    _answer_type: PhantomData<A>,
}
impl<P, S, Q, A> Queryable<P, S, Q, A> {
    pub fn new(parameters: P, initial_state: S, transition: impl Fn(&P, &S, &Q) -> (S, A) + 'static) -> Self {
        Self { parameters, state: initial_state, _query_type: PhantomData, _answer_type: PhantomData, transition: Box::new(transition) }
    }
    pub fn eval(&mut self, query: &Q) -> A {
        let (new_state, answer) = (self.transition)(&self.parameters, &self.state, query);
        self.state = new_state;
        answer
    }
}

pub struct InteractiveMeasurement<I, O, P, S, Q> {
    function: Box<dyn Fn(I) -> Queryable<P, S, Q, O>>,
}
impl<I, O, P, S, Q> InteractiveMeasurement<I, O, P, S, Q> {
    pub fn new(function: impl Fn(I) -> Queryable<P, S, Q, O> + 'static) -> Self {
        Self { function: Box::new(function) }
    }
    pub fn eval(&self, input: I) -> Queryable<P, S, Q, O> {
        (self.function)(input)
    }
}

pub fn make_some_non_interactive<I>() -> Measurement<I, I> {
    todo!()
}

pub fn make_plain_adaptive_composition<I, O>(budget: Privacy) -> InteractiveMeasurement<I, O, I, Privacy, Measurement<I, O>> {
    let function = move |data| {
        let parameters = data;
        let initial_state = budget;
        let transition = |parameters: &I, state: &Privacy, query: &Measurement<I, O>| -> (Privacy, O) {
            let new_state = state - query.privacy_loss;
            if new_state < 0.0 {
                panic!("Not enough privacy budget left!!!")
            }
            let answer = query.eval(parameters);
            (new_state, answer)
        };
        Queryable::new(parameters, initial_state, transition)
    };
    InteractiveMeasurement::new(function)
}

pub fn make_sequential_adaptive<I, O, Q>(budget: Privacy) -> InteractiveMeasurement<I, Queryable<I, Privacy, Measurement<I, O>, O>, I, Privacy, InteractiveMeasurement<I, O, I, Privacy, Q>> {
    let function = move |data| {
        let parameters = data;
        let initial_state = budget;
        let transition = |parameters: &I, state: &Privacy, query: &Measurement<I, O>| -> (Privacy, O) {
            let new_state = state - query.privacy_loss;
            if new_state < 0.0 {
                panic!("Not enough privacy budget left!!!")
            }
            let answer = query.eval(parameters);
            (new_state, answer)
        };
        Queryable::new(parameters, initial_state, transition)
    };
    InteractiveMeasurement::new(function)
}

pub fn make_concurrent_composition<I, O>() -> InteractiveMeasurement<I, O, (), (), InteractiveMeasurement<I, O, (), (), Measurement<I, O>>> {
    todo!()
}
