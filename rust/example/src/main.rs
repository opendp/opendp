use opendp::error::Fallible;
use opendp::dom::AllDomain;
use opendp::meas::{make_base_laplace, make_base_gaussian};
use opendp::sarus::{FDifferentialPrivacy, PLDistribution, make_plm};
use opendp::chain::make_basic_composition;

fn main() -> Fallible<()> {
    println!("OpenDP example");

    let priv_loss_dist = PLDistribution::new(&[(0.5, 1.0), (0.5, 1.0)]);
    let priv_loss_meas = make_plm(&[(0.00, 0.01), (0.5, 0.5), (2., 0.5)]).unwrap();
    let priv_loss_comp_meas = make_basic_composition(&priv_loss_meas, &priv_loss_meas);

    println!("{:#?}", priv_loss_meas.f() );

    laplace_example();
    gaussian_comp_example();
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

fn gaussian_comp_example() -> Fallible<()> {
    println!("Laplace and Gauussian example");
    let meas_0 = make_base_gaussian::<AllDomain<f64>>(1.0)?;
    let meas_1 = make_base_gaussian::<AllDomain<f64>>(1.0)?;
    println!("meas_0 {:?}", (meas_0.privacy_relation.relation)(&0.1, &(1., 0.01)));
    println!("meas_1 {:?}", (meas_1.privacy_relation.relation)(&0.1, &(1., 0.01)));

    let comp_meas = make_basic_composition(&meas_0, &meas_1)?;
    println!("Composed {:?}", (comp_meas.privacy_relation.relation)(&0.1, &(1., 0.02)));
    return Ok(());
}