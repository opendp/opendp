use crate::{
    error::Fallible,
    measures::{
        ApproxDPPoint,
        curves::{check_epsilon, logspace::check_delta},
    },
    traits::CInterval,
};

impl ApproxDPPoint {
    pub fn build((epsilon, delta): (f64, f64)) -> Fallible<Self> {
        check_epsilon(epsilon)?;
        check_delta(delta)?;

        let epsilon = CInterval::point(epsilon)?;
        Ok(Self {
            epsilon: epsilon.upper_f64()?,
            delta,
        })
    }
}

pub fn delta_via_approxDP(points: &[ApproxDPPoint], epsilon: f64) -> Fallible<f64> {
    let idx = points.partition_point(|point| point.epsilon <= epsilon);
    Ok(if idx == 0 { 1.0 } else { points[idx - 1].delta })
}

pub fn epsilon_via_approxdp(points: &[ApproxDPPoint], delta: f64) -> Fallible<f64> {
    check_delta(delta)?;

    if delta == 1.0 {
        return Ok(0.0);
    }

    let idx = points.partition_point(|point| point.delta > delta);

    Ok(if idx == points.len() {
        f64::INFINITY
    } else {
        points[idx].epsilon
    })
}
