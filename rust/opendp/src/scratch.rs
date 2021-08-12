#[allow(unused_variables)]
use std::marker::PhantomData;
use crate::core::{Domain, PrivacyRelation, Measure, Metric, Function};
use crate::error::Fallible;


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

pub struct Queryable<S, Q, A> {
    // parameters: P,
    state: S,
    transition: Box<dyn Fn(&S, &Q) -> (S, A)>,
    // IGNORE THE FOLLOWING, RUST IMPL QUIRK.
    _query_type: PhantomData<Q>,
    _answer_type: PhantomData<A>,
}
impl<S, Q, A> Queryable<S, Q, A> {
    pub fn new(
        // parameters: P,
        initial_state: S, transition: impl Fn(&P, &S, &Q) -> (S, A) + 'static) -> Self {
        Self {
            // parameters,
            state: initial_state, _query_type: PhantomData, _answer_type: PhantomData, transition: Box::new(transition) }
    }
    pub fn eval(&mut self, query: &Q) -> A {
        let (new_state, answer) = (self.transition)(&self.parameters, &self.state, query);
        self.state = new_state;
        answer
    }
}


pub struct InteractiveMeasurement2<DI: Domain, DO: Domain, MI: Metric, MO: Measure, S, Q, A> {
    pub input_domain: DI,
    pub output_domain: DO,
    pub function: Function<DI, Queryable<S, Q, A>>,
    pub input_metric: MI,
    pub output_measure: MO,
    pub privacy_relation: PrivacyRelation<MI, MO>,
}

type Measurement2<DI, DO, MI, MO> = InteractiveMeasurement2<DI, DO, MI, MO, (), <DI as Domain>::Carrier, <DO as Domain>::Carrier>;

// fn make_non_interactive_data_in_state<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
//     input_domain: DI,
//     output_domain: DO,
//     function: impl Fn(&DI::Carrier) -> DO::Carrier,
//     input_metric: MI,
//     output_measure: MO,
//     privacy_relation: PrivacyRelation<MI, MO>
// ) -> Measurement2<DI, DO, MI, MO> {
//     let transition = |state: DI::Carrier, query: ()| -> (DI::Carrier, DO::Carrier) {
//         function(state)
//     };
//     let outer_function = |arg: &DI::Carrier| -> Queryable<DI::Carrier, (), DO::Carrier> {
//         Queryable::new(arg.clone(), transition)
//     };
//     Measurement2 {
//         input_domain,
//         output_domain,
//         function: Function::new_fallible(outer_function),
//         input_metric,
//         output_measure,
//         privacy_relation
//     }
// }
//
// fn make_non_interactive_data_in_query<DI: Domain, DO: Domain, MI: Metric, MO: Measure>(
//     input_domain: DI,
//     output_domain: DO,
//     function: impl Fn(&DI::Carrier) -> DO::Carrier,
//     input_metric: MI,
//     output_measure: MO,
//     privacy_relation: PrivacyRelation<MI, MO>
// ) -> Measurement2<DI, DO, MI, MO> {
//     let transition = |state: (), query: DI::Carrier| -> (DI::Carrier, DO::Carrier) {
//         function(query)
//     };
//     let outer_function = |arg: &DI::Carrier| -> Queryable<(), DI::Carrier, DO::Carrier> {
//         Queryable::new((), transition)
//     };
//     Measurement2 {
//         input_domain,
//         output_domain,
//         function: Function::new_fallible(outer_function),
//         input_metric,
//         output_measure,
//         privacy_relation
//     }
// }



// fn invoke1<DI, DO, MI, MO>(this: Measurement2<DI, DO, MI, MO>, data: &<DI as Domain>::Carrier) -> <DO as Domain>::Carrier {
//     // STORE DATA IN STATE:
//     this.function.eval(data)(())
// }

impl<DI: Domain, DO: Domain, MI: Metric, MO: Measure> Measurement2<DI, DO, MI, MO> {
    fn invoke1(&self, arg: &DI::Carrier) -> Fallible<DO::Carrier> {
        (self.function.eval(&()).transition)(arg)
    }

    fn new(
        input_domain: DI,
        output_domain: DO,
        function: impl Fn(&DI::Carrier) -> DO::Carrier,
        input_metric: MI,
        output_measure: MO,
        privacy_relation: PrivacyRelation<MI, MO>
    ) -> Self {
        let transition = |state: (), query: DI::Carrier| -> (DI::Carrier, DO::Carrier) {
            function(query)
        };
        let outer_function = |arg: &DI::Carrier| -> Queryable<(), DI::Carrier, DO::Carrier> {
            Queryable::new((), transition)
        };
        Measurement2 {
            input_domain,
            output_domain,
            function: Function::new_fallible(outer_function),
            input_metric,
            output_measure,
            privacy_relation
        }
    }
}

pub struct InteractiveMeasurement<DI: Domain, DO: Domain, P, S, Q> {
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
