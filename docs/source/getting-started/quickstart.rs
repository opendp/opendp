#!/usr/bin/env -S cargo +nightly -Zscript
```cargo
# init
[dependencies]
opendp = { version = "0.9.2", features = ["contrib", "honest-but-curious"] }
# /init
```

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