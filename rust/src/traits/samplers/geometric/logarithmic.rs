use crate::{
    traits::samplers::{SampleStandardBernoulli, SampleUniformInt},
    traits::FloatBits,
};

use std::ops::{Add, Sub, AddAssign};

use num::{One, Zero};

use crate::error::Fallible;


pub fn sample_geometric_log_time<T, S>(shift: T, positive: bool, prob: S) -> Fallible<T>
where
    S: FloatBits + num::Float,
    S::Bits: Add<Output = S::Bits> + Sub<Output = S::Bits> + Zero + One + PartialOrd + AddAssign,
{
    let (num, den_pow) = as_fraction(prob)?;

    let d = S::Bits::zero();

    while sampleOnePToN(num, den_pow, 0, 2)? {
        d += S::Bits::one();
    }

    loop {
        let m = sampleOnePToM(num, den_pow)?;
        if m >= 0 {
            return (d << den_pow) + m;
        }
    }
}

pub fn sample_geometric_log_time_u64(shift: u64, positive: bool, prob: f64) -> Fallible<u64> {
    let (num, den_pow) = as_fraction(prob)?;

    let mut d = 0;

    while sampleOnePToN(num, den_pow, 0, 2)? {
        d += 1;
    }

    loop {
        let m = sampleOnePToM(num, den_pow)?;
        if m >= 0 {
            return Ok((d << den_pow) + m);
        }
    }
}

// Yannis Manolopoulos. 2002. "Binomial coefficient computation:
// recursion or iteration?", SIGCSE Bull. 34, 4 (December 2002),
// 65–67. DOI: https://doi.org/10.1145/820127.820168
// https://github.com/peteroupc/peteroupc.github.io/blob/3b4f653884eb4f47fe92f6c6b6c14f7b829507c6/randomgen.py#L2087-L2096
fn binco(n: u64, k: u64) -> u64 {
    let mut vnum = 1;
    let mut vden = 1;

    (n - k + 1..n + 1).for_each(|i| {
        vnum *= i;
        vden *= n - i + 1;
    });

    vnum / vden
}

fn sampleOnePToN(px: u64, k: u64, mut r: u64, mut i: u64) -> Fallible<bool> {
    // Returns 1 with probability (1-px/py)^n, where
    // n*p <= 1, using given random bits r of length log2(i)-1.
    let py = 1 << k;
    let mut pnum = 1;
    let mut pden = 1;

    let mut qnum = px; // px ** j
    let mut qden = py; // py ** j
    let mut j = 1;
    loop {
        if j <= k {
            let bco = binco(k, j);
            // Add a summand, namely bco * (-px / py)**j
            pnum *= qden;
            if j % 2 == 0 {
                // Even
                pnum += pden * bco * qnum;
            } else {
                // Odd
                pnum -= pden * bco * qnum;
            }
            pden *= qden;
            qnum *= px;
            qden *= py;
            j += 1;
        }
        if j > 2 || j > k {
            r = (r << 1) + bool::sample_standard_bernoulli()? as u64;
            let bk = (pnum * i) / pden;
            if r <= bk - 2 {
                return Ok(true);
            }
            if r >= bk + 1 {
                return Ok(false);
            }
            i <<= 1
        }
    }
}

fn sampleOnePToM(px: u64, k: u64) -> Fallible<u64> {
    let py = 1 << k;
    // With probability (1-px/py)^m, returns m, where
    // m is uniform in [0, 2^k) and (2^k)*p <= 1.
    // Otherwise, returns -1.
    let mut r = 0;
    let mut m = 0;

    for b in 1..k {
        m |= (bool::sample_standard_bernoulli()? as u64) << (k - b);
        // Sum b+2 summands of the binomial equivalent
        // of the probability, using high bits of m
        let mut pnum = 1;
        let mut pden = 1;
        let mut qnum = px;
        let mut qden = py;
        let mut j = 1;
        while j <= m && j <= b + 2 {
            // for j in range(1, min(m+1,b+2+1)):
            let bco = binco(m, j);
            // Add a summand, namely bco*(-px/py)**j
            if j % 2 == 0 {
                // Even
                pnum = pnum * qden + pden * bco * qnum
            } else {
                // Odd
                pnum = pnum * qden - pden * bco * qnum
            }
            pden *= qden;
            qnum *= px;
            qden *= py;
            j += 1;
        }
        r = (r << 1) + (bool::sample_standard_bernoulli()? as u64);
        let bk = pnum * (1 << b) / pden;
        if r <= bk - 2 {
            m |= u64::sample_uniform_int_0_u(1 << (k - b))?;
            return Ok(m);
        }
        if r >= bk + 1 {
            return fallible!(FailedFunction, "sampling failed");
        }
    }
    m |= bool::sample_standard_bernoulli()? as u64;
    // All of m was sampled, so calculate whole probability
    sampleOnePToN(px, m, r, 1 << k).map(|_| m)
}

/// Decomposes float number into (p_star, k) where p_star / 2^k is the smallest fraction greater than `x`.
///
/// # Arguments
/// * `x` - The value to decompose into a fraction
pub fn as_fraction<T>(x: T) -> Fallible<(T::Bits, T::Bits)>
where
    T: FloatBits + num::Float,
    T::Bits: Add<Output = T::Bits> + Sub<Output = T::Bits> + Zero + One + PartialOrd,
{
    if x.is_sign_negative() {
        return fallible!(
            FailedFunction,
            "get_smallest_greater_or_equal_power_of_two must have a positive argument"
        );
    }
    let (exponent, mantissa) = (x.exponent(), x.mantissa());

    // add implicit bit to numerator (could also use BitOr)
    let numerator = mantissa + (T::Bits::one() << T::MANTISSA_BITS);

    //
    let denominator_pow = abs_diff(
        exponent,
        T::EXPONENT_PROB + T::Bits::one() + T::MANTISSA_BITS,
    );
    Ok((numerator, denominator_pow))
}

fn abs_diff<T: Sub<Output = T> + PartialOrd>(slf: T, other: T) -> T {
    if slf < other {
        other - slf
    } else {
        slf - other
    }
}

#[cfg(test)]
mod test_geometric {
    use super::*;
    #[test]
    fn test_as_fraction() -> Fallible<()> {
        let (numer, denom_pow) = as_fraction(0.3467f64)?;
        println!("{} {}", numer, denom_pow);
        println!("{:?}", numer as f64 / 2f64.powf(denom_pow as f64));

        Ok(())
    }

    #[test]
    fn test_sample_geometric_log_time_u64() -> Fallible<()> {
        let v = sample_geometric_log_time_u64(0, true, 0.5)?;
        println!("{:?}", v);
        Ok(())
    }
}
