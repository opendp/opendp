use core::f64;

use super::*;
use crate::{
    domains::VectorDomain, metrics::SymmetricDistance, traits::InfCast,
    transformations::make_count_by,
};

#[test]
fn test_count_by_threshold() -> Fallible<()> {
    let max_influence = 1;
    let sensitivity = max_influence as f64;
    let epsilon = 2.;
    let delta = 1e-6;
    let scale = sensitivity / epsilon;
    let threshold =
        u32::inf_cast((max_influence as f64 / (2. * delta)).ln() * scale + max_influence as f64)?;
    println!("{:?}", threshold);

    let t_count = make_count_by(
        VectorDomain::new(AtomDomain::default()),
        SymmetricDistance::default(),
    )?;
    let (dom, met) = t_count.output_space();
    let m_noise = make_laplace_threshold(dom, met, scale, threshold, None)?;
    let m_count = (t_count >> m_noise)?;
    let ret = m_count.invoke(&vec![
        'a', 'a', 'a', 'a', 'a', 'a', 'a', 'a', 'b', 'b', 'b', 'b', 'b', 'b', 'b',
    ])?;
    println!("stability eval: {:?}", ret);

    let epsilon_p = m_count.map(&max_influence)?.0;
    assert_eq!(epsilon_p, epsilon);
    Ok(())
}
