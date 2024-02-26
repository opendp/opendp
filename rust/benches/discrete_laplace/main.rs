use criterion::{criterion_group, criterion_main, Criterion};
use opendp::{
    domains::{AtomDomain, VectorDomain},
    measurements::{make_vector_integer_laplace_cks20, make_vector_integer_laplace_linear},
    metrics::L1Distance,
};

pub fn collect(c: &mut Criterion) {
    (1..20).for_each(|v| {
        let scale = v as f64;
        c.bench_function(format!("{} linear", scale).as_str(), |b| {
            b.iter(|| {
                let meas = make_vector_integer_laplace_linear(
                    VectorDomain::new(AtomDomain::default()),
                    L1Distance::default(),
                    scale,
                    None,
                )
                .unwrap();
                meas.invoke(&vec![0; 1000]).unwrap();
            })
        });

        c.bench_function(format!("{} cks20", scale).as_str(), |b| {
            b.iter(|| {
                let meas = make_vector_integer_laplace_cks20(
                    VectorDomain::new(AtomDomain::default()),
                    L1Distance::default(),
                    scale,
                )
                .unwrap();
                meas.invoke(&vec![0; 1000]).unwrap();
            })
        });
    });
}

criterion_group!(benches, collect);
criterion_main!(benches);
