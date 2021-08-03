use opendp::error::Fallible;
use opendp::dom::AllDomain;
use opendp::meas::{LaplaceDomain, make_base_laplace};

fn main() -> Fallible<()> {
    println!("OpenDP example");
    let measurement = make_base_laplace::<AllDomain<f64>>(1.0)?;
    let _ret = measurement.function.eval(&0.0)?;
    println!("{:?}", _ret);

    println!("{:?}", (measurement.privacy_relation.relation)(&1.0, &0.01));
    return Ok(());
}
