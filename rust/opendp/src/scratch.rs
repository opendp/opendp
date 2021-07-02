#[allow(unused_variables)]
use std::marker::PhantomData;


pub struct Measurement<DI, DO>(PhantomData<DI>, PhantomData<DO>);

impl<DI, DO> Measurement<DI, DO> {
    pub fn eval(_data: &DI) -> DO {
        unimplemented!()
    }
}

pub fn make_sum<DI>() -> Measurement<Vec<DI>, DI> {
    todo!()
}


pub struct Queryable<Q, A>(PhantomData<Q>, PhantomData<A>);
impl<Q, A> Queryable<Q, A> {
    pub fn eval(&self, _query: &Q) -> A {
        unimplemented!()
    }
}


pub struct InteractiveMeasurement<DI, DO, Q>(PhantomData<DI>, PhantomData<DO>, PhantomData<Q>);

impl<DI, DO, Q> InteractiveMeasurement<DI, DO, Q> {
    pub fn eval(&self, _arg: &DI) -> Queryable<Q, DO> {
        unimplemented!()
    }
}


fn make_plain_adaptive<DI, DO>() -> InteractiveMeasurement<DI, DO, Measurement<DI, DO>> {
    todo!()
}

fn make_parallel_adaptive<DI, DO>() -> InteractiveMeasurement<DI, DO, InteractiveMeasurement<DI, DO, Measurement<DI, DO>>> {
    todo!()
}

fn make_sequential_adaptive<DI, DO, Q>() -> InteractiveMeasurement<DI, DO, InteractiveMeasurement<DI, DO, Q>> {
    todo!()
}

#[cfg(test)]
#[test]
fn scratch() {
    let data = vec![1.0, 2.0, 3.0];

    let im = make_plain_adaptive::<Vec<f64>, f64>();
    let queryable = im.eval(&data);
    let query1 = make_sum();
    let _answer = queryable.eval(&query1);
    let query2 = make_sum();
    let _answer = queryable.eval(&query2);

    let im = make_parallel_adaptive::<Vec<f64>, f64>();
    let query1 = make_plain_adaptive::<Vec<f64>, f64>();
    let answer1 = query1.eval(&data);
    let query1_1 = make_sum();
    let answer1_1 = answer1.eval(&query1_1);

}
