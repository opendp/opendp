use super::*;
use crate::{domains::VectorDomain, metrics::SymmetricDistance, transformations::make_count_by};

#[test]
#[cfg(feature = "partials")]
fn test_count_by_ptr() -> Fallible<()> {
    let max_influence = 1;
    let sensitivity = max_influence as f64;
    let epsilon = 2.;
    let delta = 1e-6;
    let scale = sensitivity / epsilon;
    let threshold = (max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64;
    println!("{:?}", threshold);

    let measurement = (make_count_by(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    )? >> then_laplace_threshold(scale, threshold, None))?;
    let ret = measurement.invoke(&vec!['a', 'b', 'a', 'a', 'a', 'a', 'b', 'a', 'a', 'a', 'a'])?;
    println!("stability eval: {:?}", ret);

    let epsilon_p = measurement.map(&max_influence)?.0;
    assert_eq!(epsilon_p, epsilon);
    Ok(())
}
