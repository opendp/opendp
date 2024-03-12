#!/usr/bin/env -S cargo +nightly -Zscript
```cargo
[dependencies]
opendp = { version = "0.9.2", features = ["contrib", "honest-but-curious"] }
```

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
