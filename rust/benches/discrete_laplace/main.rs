use criterion::{criterion_group, criterion_main, Criterion};
use opendp::{
    domains::{AllDomain, VectorDomain},
    measurements::{make_base_discrete_laplace_cks20, make_base_discrete_laplace_linear},
};

pub fn collect(c: &mut Criterion) {

    (1..20).for_each(|v| {
        let scale = v as f64;
        c.bench_function(format!("{} linear", scale).as_str(), |b| {
            b.iter(|| {
                let meas =
                    make_base_discrete_laplace_linear::<VectorDomain<AllDomain<i32>>, _>(scale, None)
                        .unwrap();
                meas.invoke(&vec![0; 1000]).unwrap();
            })
        });

        c.bench_function(format!("{} cks20", scale).as_str(), |b| {
            b.iter(|| {
                let meas =
                    make_base_discrete_laplace_cks20::<VectorDomain<AllDomain<i32>>, _>(scale).unwrap();
                meas.invoke(&vec![0; 1000]).unwrap();
            })
        });
    });
}

criterion_group!(benches, collect);
criterion_main!(benches);
