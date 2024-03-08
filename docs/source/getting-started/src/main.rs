use opendp::{
    domains::AtomDomain,
    measurements::then_base_laplace,
    metrics::AbsoluteDistance,
};

fn main() {
    let space = (AtomDomain::default(), AbsoluteDistance::default());
    let base_laplace = space >> then_base_laplace(1.0, None);
    let dp_value = base_laplace.expect("unexpected error").invoke(&123.0);
    println!("DP value: {}", dp_value.expect("unexpected error"));
}
