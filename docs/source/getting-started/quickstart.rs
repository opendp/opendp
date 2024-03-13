#!/usr/bin/env -S cargo +nightly -Zscript
```cargo
[dependencies]
opendp = { path = "../../../rust/", features = ["contrib", "honest-but-curious"] }
```

/*
Any other Rust examples will only have the cargo block above,
but for quickstart we want to also give an example that will work for the user.

# init
[dependencies]
opendp = { features = ["contrib", "honest-but-curious"] }
# Optionally pin the version number.
# /init
*/

// demo
use opendp::{
    domains::AtomDomain,
    error::Fallible,
    measurements::then_laplace,
    metrics::AbsoluteDistance,
};

fn main() -> Fallible<()> {
    let space = (AtomDomain::default(), AbsoluteDistance::default());
    let laplace_mechanism = (space >> then_laplace(1.0))?;
    let dp_value = laplace_mechanism.invoke(&123.0)?;
    println!("DP value: {}", dp_value);
    Ok(())
}
// /demo