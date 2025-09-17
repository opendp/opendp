// demo
use opendp::{
    domains::AtomDomain,
    error::Fallible,
    measurements::then_laplace,
    metrics::AbsoluteDistance,
};

fn main() -> Fallible<()> {
    let space = (AtomDomain::<f64>::new_non_nan(), AbsoluteDistance::<f64>::default());
    let laplace_mechanism = (space >> then_laplace(1.0, None))?;
    let dp_value = laplace_mechanism.invoke(&123.0)?;
    println!("DP value: {}", dp_value);
    Ok(())
}
// /demo