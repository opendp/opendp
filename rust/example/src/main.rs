use opendp::error::Fallible;
use opendp::dom::AllDomain;
use opendp::meas::{make_base_laplace, make_base_gaussian};


fn main() -> Fallible<()> {
    println!("OpenDP example");

    laplace_example();
    laplace_and_gaussian_example();
    return Ok(());
}

fn laplace_example() -> Fallible<()> {
    println!("Laplace example");
    let measurement = make_base_laplace::<AllDomain<f64>>(1.0)?;
    let _ret = measurement.function.eval(&0.0)?;
    println!("{:?}", _ret);
    println!("{:?}", (measurement.privacy_relation.relation)(&1.0, &0.01));
    return Ok(());
}

fn laplace_and_gaussian_example() -> Fallible<()> {
    println!("Laplace and Gauussian example");
    let laplace_meas = make_base_laplace::<AllDomain<f64>>(1.0)?;
    let gaussian_meas = make_base_gaussian::<AllDomain<f64>>(1.0)?;
    println!("Laplace {:?}", (laplace_meas.privacy_relation.relation)(&1.0, &1.));
    println!("Gaussian {:?}", (gaussian_meas.privacy_relation.relation)(&0.1, &(1., 0.01)));
    return Ok(());
}