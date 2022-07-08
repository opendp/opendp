mod bernoulli;
pub use bernoulli::*;

mod continuous;
pub use continuous::*;

mod geometric;
pub use geometric::*;

mod uniform;
pub use uniform::*;

use rand::RngCore;
use rand::prelude::SliceRandom;
#[cfg(feature="use-mpfr")]
use rug::rand::ThreadRandGen;


use crate::error::Fallible;
#[cfg(any(not(feature="use-mpfr"), not(feature="use-openssl")))]
use rand::Rng;

#[cfg(feature="use-openssl")]
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    use openssl::rand::rand_bytes;
    if let Err(e) = rand_bytes(buffer) {
        fallible!(FailedFunction, "OpenSSL error: {:?}", e)
    } else { Ok(()) }
}

#[cfg(not(feature="use-openssl"))]
pub fn fill_bytes(buffer: &mut [u8]) -> Fallible<()> {
    if let Err(e) = rand::thread_rng().try_fill(buffer) {
        fallible!(FailedFunction, "Rand error: {:?}", e)
    } else { Ok(()) }
}

pub struct GeneratorOpenDP {
    pub error: Fallible<()>,
}

impl GeneratorOpenDP {
    pub fn new() -> Self {
        GeneratorOpenDP { error: Ok(()) }
    }
}

#[cfg(feature="use-mpfr")]
impl ThreadRandGen for GeneratorOpenDP {
    fn gen(&mut self) -> u32 {
        let mut buffer = [0u8; 4];
        if let Err(e) = fill_bytes(&mut buffer) {
            self.error = Err(e)
        }
        u32::from_ne_bytes(buffer)
    }
}

impl RngCore for GeneratorOpenDP {
    fn next_u32(&mut self) -> u32 {
        let mut buffer = [0u8; 4];
        self.fill_bytes(&mut buffer);
        u32::from_ne_bytes(buffer)
    }

    fn next_u64(&mut self) -> u64 {
        let mut buffer = [0u8; 8];
        self.fill_bytes(&mut buffer);
        u64::from_ne_bytes(buffer)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        if let Err(e) = fill_bytes(dest) {
            self.error = Err(e)
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        fill_bytes(dest).map_err(rand::Error::new)
    }
}

pub trait Shuffle {
    fn shuffle(&mut self) -> Fallible<()>;
}
impl<T> Shuffle for Vec<T> {
    fn shuffle(&mut self) -> Fallible<()> {
        let mut rng = GeneratorOpenDP::new();
        SliceRandom::shuffle(self.as_mut_slice(), &mut rng);
        rng.error
    }
}


#[cfg(test)]
mod test_utils {
    use std::fmt::Debug;
    use std::iter::Sum;
    use std::ops::{Div, Sub};

    use num::traits::real::Real;
    use statrs::function::erf;

    use num::{NumCast, One};

    /// returns z-statistic that satisfies p == ∫P(x)dx over (-∞, z),
    ///     where P is the standard normal distribution
    pub fn normal_cdf_inverse(p: f64) -> f64 {
        std::f64::consts::SQRT_2 * erf::erfc_inv(2.0 * p)
    }

    macro_rules! c {($expr:expr; $ty:ty) => ({let t: $ty = NumCast::from($expr).unwrap(); t})}

    pub fn test_proportion_parameters<T, FS: Fn() -> T>(sampler: FS, p_pop: T, alpha: f64, err_margin: T) -> bool
        where T: Sum<T> + Sub<Output=T> + Div<Output=T> + Real + Debug + One {

        // |z_{alpha/2}|
        let z_stat = c!(normal_cdf_inverse(alpha / 2.).abs(); T);

        // derived sample size necessary to conduct the test
        let n: T = (p_pop * (T::one() - p_pop) * (z_stat / err_margin).powi(2)).ceil();

        // confidence interval for the mean
        let abs_p_tol = z_stat * (p_pop * (T::one() - p_pop) / n).sqrt(); // almost the same as err_margin

        println!("sampling {:?} observations to detect a change in proportion with {:.4?}% confidence",
                 c!(n; u32), (1. - alpha) * 100.);

        // take n samples from the distribution, compute average as empirical proportion
        let p_emp: T = (0..c!(n; u32)).map(|_| sampler()).sum::<T>() / n;

        let passed = (p_emp - p_pop).abs() < abs_p_tol;

        println!("stat: (tolerance, pop, emp, passed)");
        println!("    proportion:     {:?}, {:?}, {:?}, {:?}", abs_p_tol, p_pop, p_emp, passed);
        println!();

        passed
    }

}