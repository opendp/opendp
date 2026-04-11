mod bernoulli;
mod gcd;
mod geometric;
mod symmetric;

use dashu::rational::RBig;

use crate::traits::samplers::test::BASE_N;

pub const N_BERNOULLI: usize = 2 * BASE_N;
pub const N_GEOM_FAST: usize = 2 * BASE_N;
pub const N_GEOM_SLOW: usize = BASE_N / 2;
pub const N_LAPLACE: usize = 2 * BASE_N;
pub const N_GAUSS: usize = 2 * BASE_N;

pub fn p_exp_neg(x: &RBig) -> f64 {
    (-x.to_f64().value()).exp()
}
