use std::convert::TryFrom;

use crate::{
    traits::{WrappingCast, SaturatingCast},
    traits::samplers::{SampleStandardBernoulli, SampleUniformInt}
};

// stands for Big Integer, an integer with unlimited precision, from gmp
use rug::{Complete, Integer, Rational};

use num::{One, Zero};

use crate::error::Fallible;

use super::Tail;

pub fn sample_geometric_log_time<T, P>(shift: T, direction: bool, prob: P, tail: Tail<T>) -> Fallible<T>
where
    Integer: From<T>,
    T: TryFrom<Integer> + WrappingCast<Integer> + SaturatingCast<Integer>,
    Rational: TryFrom<P>,
{
    let mut shift = Integer::from(shift);
    let prob = Rational::try_from(prob).map_err(|_| err!(FailedFunction, "prob must be finite"))?;

    let (numer, denom) = prob.into_numer_denom();
    let geo = sample_standard_geometric_log_time(numer, denom)?;
    if direction {
        shift += geo;
    } else {
        shift -= geo;
    }
    Ok(match tail {
        Tail::Censored(_) => T::saturating_cast(shift),
        Tail::Modular => T::wrapping_cast(shift),
    })
}

fn sample_standard_geometric_log_time(px: Integer, py: Integer) -> Fallible<Integer> {
    // https://github.com/peteroupc/peteroupc.github.io/blob/14c3e050998271f163fd7e896fb2b9fe1cfc15f9/randomgen.py#L2257-L2275
    // Implements the algorithm given in Bringmann, K.
    // and Friedrich, T., 2013, July. Exact and efficient generation
    // of geometric random variates and random graphs, in
    // _International Colloquium on Automata, Languages, and
    // Programming_ (pp. 267-278).

    // Algorithm runs as follows:
    // 1. Segment Z+ (the positive integers) into segments of length 2^k
    // 2. Sample
    // Partition Z+ into segments of length 2^k.
    // Calculate k for px/py
    let mut k = 0u32;
    let mut pn = px.clone();
    while pn.clone() << 1 <= py {
        k += 1;
        pn <<= 1;
    }
    // Identify which segment of length 2^k is selected (segment d)
    let mut d = Integer::zero();
    while sampleOnePToN(px.clone(), py.clone(), 1 << k, 0, 2)? {
        d += 1;
    }

    // Sample within the d+1th segment
    // Use rejection sampling. sampleOnePToM is a sample from the censored geometric to [0, k]
    let offset = loop {
        if let (Some(m), _) = sampleOnePToM(px.clone(), py.clone(), k.into())? {
            break m;
        }
    };

    Ok((d << k) + offset)
}

// // https://github.com/peteroupc/peteroupc.github.io/blob/master/randomgen.py#L2181-L2228
// pub fn sample_geometric_log_time_truncated<T: Integer, P>(
//     shift: T,
//     positive: bool,
//     prob: P,
// ) -> Fallible<T>
//     where Rational: TryFrom<P> {
//     let prob = Rational::try_from(prob).map_err(|e| err!(FailedFunction, "prob must be finite"))?;

//     let (numer, denom): (Integer, Integer) = prob.into_numer_denom();
//     let mut k = 0;
//     let mut pn = numer;
//     loop {
//         pn <<= 2;
//         if pn <= denom {
//             break
//         }
//         k += 1;
//     }

//     let mut d = Integer::zero();
//     while sampleOnePToN(numer.clone(), denom.clone(), 1 << k, 0.into(), 2.into())? {
//         d += 1;
//         if (d << k) >= m2 {
//             return Ok(n)
//         }
//     }

//     loop {
//         if let (mut m, Some(mut mbit)) = sampleOnePToM(numer.clone(), denom.clone())? {
//             while mbit >= 0 {
//                 let b = bool::sample_standard_bernoulli()?;
//                 m |= (b as u32) << mbit;
//                 mbit -= 1;
//                 if b {
//                     break
//                 }
//             }
//             if (d << k) + m >= m2 {
//                 return n
//             }
//             m += u32::sample_uniform_int_0_u(1 << (mbit + 1));
//             break n.min((d << k) + m);
//         }
//     }
// }

// Yannis Manolopoulos. 2002. "Binomial coefficient computation:
// recursion or iteration?", SIGCSE Bull. 34, 4 (December 2002),
// 65–67. DOI: https://doi.org/10.1145/820127.820168
// https://github.com/peteroupc/peteroupc.github.io/blob/3b4f653884eb4f47fe92f6c6b6c14f7b829507c6/randomgen.py#L2087-L2096
fn binco(n: u128, k: u128) -> Integer {
    let mut vnum = Integer::one();
    let mut vden = Integer::one();

    for i in n - k + 1..n + 1 {
        vnum *= i;
        vden *= n - i + Integer::one();
    }

    vnum / vden
}

// https://github.com/peteroupc/peteroupc.github.io/blob/14c3e050998271f163fd7e896fb2b9fe1cfc15f9/randomgen.py#L2098-L2130
fn sampleOnePToN(px: Integer, py: Integer, n: u128, mut r: u128, mut i: u128) -> Fallible<bool> {
    // Returns 1 with probability (1-px/py)^n, where
    // n*p <= 1, using given random bits r of length log2(i)-1.
    let mut pnum = Integer::one();
    let mut pden = Integer::one();

    let mut qnum = px.clone();
    let mut qden = py.clone();
    let mut j = 1;
    loop {
        if j <= n {
            let bco = binco(n, j);
            // Add a summand, namely bco * (-px / py)**j
            pnum *= &qden;
            if j % 2 == 0 {
                // Even
                pnum += &pden * bco * &qnum;
            } else {
                // Odd
                pnum -= &pden * bco * &qnum;
            }
            pden *= &qden;
            qnum *= &px;
            qden *= &py;
            j += 1;
        }
        if j > 2 || j > n {
            r <<= 1;
            if bool::sample_standard_bernoulli()? {
                r += 1;
            }
            let bk: Integer = (&pnum * &i).complete() / &pden;
            if r <= bk.clone() - 2 {
                return Ok(true);
            }
            if r >= bk + 1 {
                return Ok(false);
            }
            i <<= 1;
        }
    }
}

// https://github.com/peteroupc/peteroupc.github.io/blob/master/randomgen.py#L2132-L2179
fn sampleOnePToM(px: Integer, py: Integer, k: u128) -> Fallible<(Option<Integer>, Option<u128>)> {
    // With probability (1-px/py)^m, returns m, where
    // m is uniform in [0, 2^k) and (2^k)*p <= 1.
    // Otherwise, returns None.
    let mut r = 0;
    let mut m = 0;

    for b in 1..k {
        // randomly set the (k-b)th bit.
        m |= (bool::sample_standard_bernoulli()? as u128) << (k - b);
        // Sum b+2 summands of the binomial equivalent
        // of the probability, using high bits of m
        let mut pnum = Integer::one();
        let mut pden = Integer::one();

        let mut qnum = px.clone();
        let mut qden = py.clone();
        let mut j = 1;
        while j <= m && j <= b + 2 {
            // for j in range(1, min(m+1,b+2+1)):
            let bco = binco(m, j);
            // Add a summand, namely bco*(-px/py)**j
            if j % 2 == 0 {
                // Even
                pnum = &pnum * &qden + &pden * bco * &qnum
            } else {
                // Odd
                pnum = &pnum * &qden - &pden * bco * &qnum
            }
            pden *= &qden;
            qnum *= &px;
            qden *= &py;
            j += 1;
        }
        r <<= 1;
        r += bool::sample_standard_bernoulli()? as u32;
        let bk: Integer = pnum * (1 << b) / pden;
        if r <= bk.clone() - 2 {
            // TODO: why does original implementation return a different value for m if returning bits?
            // https://github.com/peteroupc/peteroupc.github.io/blob/14c3e050998271f163fd7e896fb2b9fe1cfc15f9/randomgen.py#L2161-L2163
            m |= u128::sample_uniform_int_0_u(1 << (k - b))?;
            return Ok((Some(m.into()), Some(k - b - 1)));
        }
        if r >= bk + 1 {
            return Ok((None, Some(0)));
        }
    }
    m |= bool::sample_standard_bernoulli()? as u128;
    // All of m was sampled, so calculate whole probability
    if sampleOnePToN(px, py, m, r.into(), 1 << k)? {
        Ok((Some(m.into()), None))
    } else {
        Ok((None, Some(0)))
    }
}

#[cfg(test)]
mod test_geometric {

    use super::*;

    #[test]
    fn test_sample_geometric_log_time() -> Fallible<()> {
        let v1 = sample_geometric_log_time(0, true, 2f64.powi(-127), Tail::Censored(None))?;
        println!("v1 {:?}", v1);

        // let v2 = sample_geometric_log_time(0, true, 0.981237419571993247197413429)?;
        // println!("v2 {:?}", v2);
        Ok(())
    }
}
