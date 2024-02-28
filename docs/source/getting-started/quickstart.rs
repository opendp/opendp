// 2-use
// TODO

use opendp::{
    domains::AtomDomain,
    measurements::then_base_laplace,
    metrics::L1Distance,
};

#[cfg(all(feature = "partials"))]
let domain = AtomDomain(f64);
let distance = L1Distance(f64);
let space = (domain, distance);
let base_laplace = space >> then_base_laplace(1.);
let dp_value = base_laplace(123.0);