//! Compute weight tables via dynamic programming for the integer partition
//! exponential mechanism.
//!
use super::bounds::PartitionBound;
use crate::{
    error::Fallible,
    measurements::b2dp::{utilities::exactarithmetic::ArithmeticConfig, Eta},
};
use rug::{ops::Pow, Float};
use std::cmp;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct Key {
    /// Candidate value
    pub q: i64,
    /// Index
    pub i: usize,
}

/// Weight table for sampling from the integer partition
/// exponential mechanism and computing.
#[derive(Debug)]
pub struct WeightTable {
    /// The weight table
    pub weights: HashMap<Key, Float>,
    /// Arithmetic config
    pub arithmetic_config: ArithmeticConfig,
    /// Coefficients table for bias computation
    pub coefficients: HashMap<Key, Float>,
    /// Total weights table for bias computation
    pub total_weights: HashMap<Key, Float>,
    /// Probabilities table for bias computation
    pub probabilities: HashMap<Key, Float>,
}

impl WeightTable {
    /// Computes the weight for a given `key` specifying an index and value and stores
    /// it in the weight table.
    ///
    /// This method is specified recursively, but is significantly more efficient
    /// if implemented without recursion. The code closely mirrors the recursive
    /// specification, but returns an error in any place a recursive call would be
    /// required if the value is not already present in the dynamic programming table.
    /// The caller is expected to call `comp_weights` in the required order:
    ///   1. `key.i` from largest to smallest.
    ///   2. `key.q` from smallest to largest for each `i`.
    ///
    /// ### Arguments
    ///   * `key`: the index and value to compute the weight for.
    ///   * `x`: the target (private) integer partition
    ///   * `pb`: the partition bounds
    ///   * `eta`: the privacy parameter
    ///
    /// ### Returns
    /// Returns a Float representing the weight or an error if the weight could not
    /// be determined. Errors occur when calls are made without filling in the required
    /// dependent values in the table.
    fn comp_weights(
        &mut self,
        key: Key,
        x: &Vec<i64>,
        pb: &PartitionBound,
        eta: Eta,
    ) -> Fallible<Float> {
        // If the value is already in the table, return it
        if let Some(w) = self.weights.get(&key) {
            return Ok(self.arithmetic_config.get_float(w));
        };

        // If q is out of bounds, return weight 0
        if key.q > pb.upper[key.i] {
            let z = self.arithmetic_config.get_float(0.0);
            self.weights.insert(key, z);
            return Ok(self.arithmetic_config.get_float(0.0));
        }

        // the i^th element in the target partition x, or a trailing zero if
        // i > x.len()
        let xi = match x.len() > key.i {
            true => x[key.i],
            false => 0,
        };

        // Check if q is the only option
        if pb.upper[key.i] == pb.lower[key.i] && pb.upper[key.i] == key.q {
            // Weight of 1 if q is the only option
            let o = self.arithmetic_config.get_float(1.0);
            self.weights.insert(key, o);
            return Ok(self.arithmetic_config.get_float(1.0));
        }

        let mut val = self.arithmetic_config.get_float(0.0);

        // Optimized case
        if key.q >= pb.lower[key.i] + 1 {
            // if q >= x_i + 1, then Delta(q,i) = 1, -1 otherwise
            let mut deltaqi = -1;
            if key.q >= xi + 1 {
                deltaqi = 1;
            }
            let base = eta.get_base(self.arithmetic_config.precision).unwrap();
            let mut k = Key {
                q: key.q - 1,
                i: key.i,
            };
            let mut w;
            if let Some(wt) = self.weights.get(&k) {
                w = wt;
            } else {
                return fallible!(FailedFunction, "Missing prior value.");
            }

            val = val + base.pow(deltaqi) * w;

            // Double check this - the way it's written in the algorithm is confusing.
            // Note: this code is never reached for i >= pb.upper.len()-1, as the
            // value of q would have been out of bounds, or equal to the terminating
            // pair.
            if pb.upper[key.i + 1] >= key.q {
                k = Key {
                    q: key.q,
                    i: key.i + 1,
                };
                if let Some(wt) = self.weights.get(&k) {
                    w = wt;
                } else {
                    return fallible!(FailedFunction, "Missing prior value.");
                }
                let base = eta.get_base(self.arithmetic_config.precision).unwrap();
                val = val + base.pow((key.q - xi).abs()) * w;
            }
        } else {
            // unoptimized case

            // Note: this code is never reached for i >= pb.upper.len()-1, as the
            // value of q would have been out of bounds, or equal to the terminating
            // pair.
            let mut qprime = pb.lower[key.i + 1];
            let bound = cmp::min(pb.upper[key.i + 1], key.q);
            let mut w;
            while qprime <= bound {
                let base = eta.get_base(self.arithmetic_config.precision).unwrap();
                let k = Key {
                    q: qprime,
                    i: key.i + 1,
                };
                if let Some(wt) = self.weights.get(&k) {
                    w = wt;
                } else {
                    return fallible!(FailedFunction, "Missing prior value.");
                }
                val = val + base.pow((key.q - xi).abs()) * w;
                qprime += 1;
            }
        }
        let result = self.arithmetic_config.get_float(&val);
        self.weights.insert(key, val);

        return Ok(result);
    }

    /// Computes the weight table for the given bounds, enforcing an exact
    /// bound if exact total specified in `exact`.
    ///
    /// ## Arguments
    ///   * `eta`: privacy parameter
    ///   * `pb`: a set of upper and lower bounds for the partition
    ///   * `x`: the target (private) integer partition
    /// ## Returns
    /// A weight table consisting of the cumulative weights for all completions
    /// beginning with value `q` at index `i`.
    ///
    /// ### Exact Arithmetic
    /// This function calls `enter_exact_scope()` and
    /// `exit_exact_scope()`, and therefore clears the `mpfr::flags` and **does not preserve the
    /// incoming flag state.**
    ///
    /// ### Known Timing Channels
    /// This method does not compute worst-case arithmetic up front, and instead increases
    /// precision as needed during computation.
    pub fn from_bounds(eta: Eta, pb: &PartitionBound, x: &Vec<i64>) -> Fallible<WeightTable> {
        // Check inputs
        pb.check()?;

        // Initialize the weight table
        let weights: HashMap<Key, Float> = HashMap::new();
        let total_weights: HashMap<Key, Float> = HashMap::new();
        let coefficients: HashMap<Key, Float> = HashMap::new();
        let probabilities: HashMap<Key, Float> = HashMap::new();
        let arithmetic_config = ArithmeticConfig::basic()?;
        let mut wt = WeightTable {
            weights,
            arithmetic_config,
            coefficients,
            total_weights,
            probabilities,
        };

        wt.arithmetic_config.enter_exact_scope()?;
        // Compute weights
        // Calls in order:
        // `i`: largest to smallest
        // `q`: smallest to largest for each `i`
        for i in (0..pb.upper.len()).rev() {
            for q in pb.lower[i]..pb.upper[i] + 1 {
                // inclusive
                let k = Key { q, i };
                wt.comp_weights(k, &x, &pb, eta)?;
                // If the weight computation resulted in inexact arithmetic,
                // increase the precision and try again.
                while ArithmeticConfig::check_mpfr_flags().is_err() {
                    // exit the exact scope
                    let _exit = wt.arithmetic_config.exit_exact_scope();
                    // reset the inexact arithmetic state
                    wt.arithmetic_config.inexact_arithmetic = false;
                    // add 16 to the precision
                    // TODO: magic number, consider setting a default increase parameter
                    wt.arithmetic_config.increase_precision(16)?;
                    // re-enter the exact scope (clears flags)
                    wt.arithmetic_config.enter_exact_scope()?;
                    // remove the incorrectly computed key
                    wt.weights.remove(&k);
                    // try again
                    wt.comp_weights(k, &x, &pb, eta)?;
                }
            }
        }
        wt.arithmetic_config.exit_exact_scope()?;
        Ok(wt)
    }

    /// DO NOT USE except for bug demonstration!
    /// Computes the weight table for the given bounds WITHOUT exact arithmetic
    /// enforcement.
    ///
    /// ## Arguments
    ///   * `eta`: privacy parameter
    ///   * `pb`: a set of upper and lower bounds for the partition
    ///   * `x`: the target (private) integer partition
    ///
    /// ## Returns
    /// DO NOT USE. A weight table consisting of the cumulative weights for all completions
    /// beginning with value `q` at index `i`, with INTENTIONALLY INEXACT ARITHMETIC for
    /// comparison and testing purposes only.
    pub fn inexact_from_bounds(
        eta: Eta,
        pb: &PartitionBound,
        x: &Vec<i64>,
    ) -> Fallible<WeightTable> {
        // Check inputs
        pb.check()?;

        // Initialize the weight table
        let weights: HashMap<Key, Float> = HashMap::new();
        let total_weights: HashMap<Key, Float> = HashMap::new();
        let coefficients: HashMap<Key, Float> = HashMap::new();
        let probabilities: HashMap<Key, Float> = HashMap::new();
        let mut arithmetic_config = ArithmeticConfig::basic()?;
        arithmetic_config.precision = 16;
        let mut wt = WeightTable {
            weights,
            arithmetic_config,
            coefficients,
            total_weights,
            probabilities,
        };

        // Compute weights
        // Calls in order:
        // `i`: largest to smallest
        // `q`: smallest to largest for each `i`
        for i in (0..pb.upper.len()).rev() {
            for q in pb.lower[i]..pb.upper[i] + 1 {
                // inclusive
                let k = Key { q, i };
                wt.comp_weights(k, &x, &pb, eta)?;
                // Precision increase logic ommitted intentionally.
            }
        }
        Ok(wt)
    }

    /// Computes the bias of sampling from the weight table.
    /// Stores helper tables of coefficients, total weights, etc,
    /// for later use if needed.
    /// ## Arguments
    ///  * `pb`: the PartitionBound to be used for sampling
    ///  * `x`: the target partition `x`. (Note that this does not have to
    ///    be the same target partition that was used to create the table.)
    ///  ## Returns
    ///  A vector indicating the bias of each index if `x` were the target partition.
    pub fn get_bias(&mut self, pb: &PartitionBound, x: &Vec<i64>) -> Fallible<Vec<f64>> {
        pb.check()?;
        // Fill in the total_weights table
        for i in 0..pb.upper.len() {
            self.compute_total_weight(i, pb);
        }

        // Fill in probability and coefficients tables

        // Compute Base-Case Probabilities
        for q in pb.lower[0]..pb.upper[0] + 1 {
            let k = Key { i: 0, q };
            self.get_probability(k, pb)?;
        }
        // Compute remaining Probabilities
        for i in 1..pb.upper.len() {
            for q in (pb.lower[i]..pb.upper[i] + 1).rev() {
                let k = Key { i, q };
                // Get the coefficients
                self.get_coefficient(k, pb)?;
            }
            for q in (pb.lower[i]..pb.upper[i] + 1).rev() {
                let k = Key { i, q };
                // Get the probabilities
                self.get_probability(k, pb)?;
            }
        }

        // Compute Bias
        let mut b: Vec<f64> = Vec::new();
        for i in 0..pb.upper.len() {
            let mut bi = self.arithmetic_config.get_float(0);
            for q in pb.lower[i]..pb.upper[i] {
                let k = Key { q, i };
                let p = self.get_probability(k, pb).unwrap();
                let xi = if i < x.len() { x[i] } else { 0 };
                let diff = xi - q;
                bi = bi + p * diff;
            }
            b.push(bi.to_f64());
        }
        Ok(b)
    }

    /// Get the probability that the mechanism outputs k.q at
    /// position k.i (P_{q,i}).
    /// TODO: fix messy unwraps
    fn get_probability(&mut self, k: Key, pb: &PartitionBound) -> Fallible<Float> {
        // Check if already computed
        if self.probabilities.contains_key(&k) {
            return Ok(self
                .arithmetic_config
                .get_float(self.probabilities.get(&k).unwrap()));
        }
        // Check if in bounds
        if k.q > pb.upper[k.i] || k.q < pb.lower[k.i] {
            let p = self.arithmetic_config.get_float(0);
            self.probabilities.insert(k, Float::with_val(p.prec(), &p));
            return Ok(p);
        }

        // Base Case
        if k.i == 0 {
            let ku = Key {
                i: k.i,
                q: pb.upper[k.i],
            };
            let tau_u_i = self.get_total_weight(ku, pb);
            let mut w = self.weights.get(&k);
            let z = self.arithmetic_config.get_float(0);
            if w.is_none() {
                // This shouldn't ever happen
                w = Some(&z);
            }
            let p = self.arithmetic_config.get_float(w.unwrap() / tau_u_i);
            self.probabilities.insert(k, Float::with_val(p.prec(), &p));
            return Ok(p);
        }

        let mut t_min = pb.lower[k.i - 1];
        if t_min < k.q {
            t_min = k.q;
        }
        let k_t_min = Key { i: k.i, q: t_min };
        let c_t_min = self.get_coefficient(k_t_min, pb).unwrap();
        let p = Float::with_val(c_t_min.prec(), c_t_min * self.weights.get(&k).unwrap());
        self.probabilities.insert(k, Float::with_val(p.prec(), &p));
        Ok(p)
    }

    /// Get the coefficient (C_{T,i})
    fn get_coefficient(&mut self, k: Key, pb: &PartitionBound) -> Fallible<Float> {
        // Check if already computed
        if self.coefficients.contains_key(&k) {
            return Ok(self
                .arithmetic_config
                .get_float(self.coefficients.get(&k).unwrap()));
        }
        // If k.q is out of bounds, no valid completions so coefficient is zero
        if k.q < pb.lower[k.i] {
            return Ok(self.arithmetic_config.get_float(0));
        }
        if k.i > 0 && k.q > pb.upper[k.i - 1] {
            return Ok(self.arithmetic_config.get_float(0));
        }

        // Base Case: k.q = upper[k.i-1]
        if k.q == pb.upper[k.i - 1] {
            let t = self.get_total_weight(k, pb);
            let km1 = Key {
                i: k.i - 1,
                q: pb.upper[k.i - 1],
            };
            let p = self.get_probability(km1, pb)?;
            let c_qi = Float::with_val(p.prec(), p / t);
            self.coefficients
                .insert(k, self.arithmetic_config.get_float(&c_qi));
            return Ok(c_qi);
        }

        let k1 = Key { q: k.q + 1, i: k.i };
        let c_q1 = self.get_coefficient(k1, pb)?;
        // Get probability P_{t, i-1}
        let km1 = Key { q: k.q, i: k.i - 1 };
        let p = self.get_probability(km1, pb)?;
        // Get total weight tau_{q,i}
        let t = self.get_total_weight(k, pb);

        let p_t = Float::with_val(p.prec(), p / t);
        let c_qi = Float::with_val(p_t.prec(), c_q1 + p_t);
        self.coefficients
            .insert(k, self.arithmetic_config.get_float(&c_qi));
        Ok(c_qi)
    }

    /// Get the total weight (\tau_{T,i})
    /// TODO: Consider adding error if weight can't be computed
    fn get_total_weight(&mut self, k: Key, pb: &PartitionBound) -> Float {
        if k.q > pb.upper[k.i] {
            let kmax = Key {
                i: k.i,
                q: pb.upper[k.i],
            };
            let wmax = self.total_weights.get(&kmax).unwrap();
            return self.arithmetic_config.get_float(wmax);
        }

        if k.q < pb.lower[k.i] {
            return self.arithmetic_config.get_float(0);
        }
        if !self.total_weights.contains_key(&k) {
            self.compute_total_weight(k.i, pb);
        }
        return self
            .arithmetic_config
            .get_float(self.total_weights.get(&k).unwrap());
    }
    /// Compute the total weight (\tau_{T,i}) of all elements in [lower_bound(i), T]
    /// for index i
    /// TODO: consider adding error if weight can't be computed for some reason
    fn compute_total_weight(&mut self, i: usize, pb: &PartitionBound) {
        let mut k = Key { q: pb.lower[i], i };
        let starting_weight = self.weights.get(&k);
        if starting_weight.is_none() {}
        let mut tau = self.arithmetic_config.get_float(0);

        for j in pb.lower[k.i]..pb.upper[k.i] + 1 {
            k = Key { q: j, i };
            let next_weight = self.weights.get(&k);
            if next_weight.is_none() {
                self.total_weights
                    .insert(k, self.arithmetic_config.get_float(0));
            } else {
                let next_total = self.arithmetic_config.get_float(tau + next_weight.unwrap());
                tau = self.arithmetic_config.get_float(&next_total);
                self.total_weights.insert(k, next_total);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_key() {
        let _k = Key { q: 10, i: 10 };
    }

    #[test]
    fn test_bias() {
        let pb = PartitionBound::with_cells(5, 6).unwrap();
        let eta = Eta::new(1, 1, 1).unwrap();
        let x: Vec<i64> = vec![5, 4, 3, 2, 1];
        let mut wt = WeightTable::from_bounds(eta, &pb, &x).unwrap();
        let b = wt.get_bias(&pb, &x);
        assert!(b.is_ok());
        println!("Bias: {:?}", b.unwrap());
        println!("Weight Table\n {:?}", wt.weights);
        println!("Total Weights\n {:?}", wt.total_weights);
        println!("Probs\n {:?}", wt.probabilities);
        println!("Coefficients \n {:?}", wt.coefficients);
        println!("Upper: {:?}", pb.upper);
        println!("Lower: {:?}", pb.lower);
    }

    #[test]
    fn test_weights() {
        let pb = PartitionBound::with_cells(5, 6).unwrap();
        let eta = Eta::new(1, 1, 1).unwrap();
        let x: Vec<i64> = vec![5, 4, 3, 2, 1];
        let wt = WeightTable::from_bounds(eta, &pb, &x);
        assert!(wt.is_ok());
        let key = Key { q: 5, i: 0 };
        let table = wt.unwrap();
        let w = table.weights.get(&key).unwrap();
        assert!(*w > 0);
        println!("{:?}", table.arithmetic_config.precision);
        //assert_eq!(*w,Float::with_val(53,0));
    }

    /// A test case that requires the precision to be increased
    #[test]
    fn test_medium_precision_weights() {
        let eta = Eta::new(1, 1, 1).unwrap();
        let n: i64 = 200;
        let mut x: Vec<i64> = (0..n).map(|x| x).rev().collect();
        for i in (n as usize / 2)..(n as usize) {
            x[i] = 0;
        }
        let total_count: i64 = x.iter().sum();
        println!("{:?}", total_count);

        let pb = PartitionBound::with_cells(total_count as usize, n as usize).unwrap();

        let wt = WeightTable::from_bounds(eta, &pb, &x);
        //if wt.is_err() {println!("{:?}", wt.err()); }
        assert!(wt.is_ok());
        let key = Key { q: 5, i: 0 };
        let table = wt.unwrap();
        let w = table.weights.get(&key).unwrap();
        assert!(*w > 0);
        println!("{:?}", table.arithmetic_config.precision);
        //println!("{:?}",pb.upper);
        //println!("{:?}",pb.lower);
    }

    /// A test case to demonstrate lost weights due to inexact arithmetic
    #[test]
    fn test_inexact_weight_difference() {
        let eta = Eta::new(/*3,2,1*/ 1, 1, 1).unwrap();
        let n: i64 = 100;
        let mut x: Vec<i64> = (0..n).map(|x| x).rev().collect();
        for i in (n as usize / 2)..(n as usize) {
            x[i] = 0;
        }
        let total_count: i64 = x.iter().sum();
        //println!("{:?}",total_count);

        let d = 10;
        let total_cells = n;
        let pb =
            PartitionBound::from_dist(d, &x, total_count as usize, total_cells as usize).unwrap();
        let wt = WeightTable::from_bounds(eta, &pb, &x).unwrap();
        let wt_inexact = WeightTable::inexact_from_bounds(eta, &pb, &x).unwrap();

        for k in wt.weights.keys() {
            println!(
                "exact, x, {:?}, {:?}, {:?}",
                k.i,
                k.q,
                wt.weights.get(&k).unwrap().to_f64()
            );
        }

        for k in wt_inexact.weights.keys() {
            println!(
                "inexact, x, {:?}, {:?}, {:?}",
                k.i,
                k.q,
                wt_inexact.weights.get(&k).unwrap().to_f64()
            );
        }

        // Swap one count
        x[0] -= 1;
        x[(n as usize) / 2] += 1;
        let pb =
            PartitionBound::from_dist(d, &x, total_count as usize, total_cells as usize).unwrap();
        let wt = WeightTable::from_bounds(eta, &pb, &x).unwrap();
        let wt_inexact = WeightTable::inexact_from_bounds(eta, &pb, &x).unwrap();

        for k in wt.weights.keys() {
            println!(
                "exact, xprime, {:?}, {:?}, {:?}",
                k.i,
                k.q,
                wt.weights.get(&k).unwrap().to_f64()
            );
        }

        for k in wt_inexact.weights.keys() {
            println!(
                "inexact, xprime, {:?}, {:?}, {:?}",
                k.i,
                k.q,
                wt_inexact.weights.get(&k).unwrap().to_f64()
            );
        }
    }

    /// A test case to demonstrate lost weights due to inexact arithmetic
    #[test]
    fn test1_1_inexact_weight_difference_1() {
        let eta = Eta::new(/*3,2,1*/ 1, 1, 1).unwrap();
        let n: i64 = 200;
        let mut x: Vec<i64> = (0..n).map(|x| x).rev().collect();
        for i in (n as usize / 2)..(n as usize) {
            x[i] = 0;
        }
        let total_count: i64 = x.iter().sum();
        println!("{:?}", total_count);

        let pb = PartitionBound::with_cells(total_count as usize, n as usize).unwrap();

        let wt = WeightTable::from_bounds(eta, &pb, &x).unwrap();
        let wt_inexact = WeightTable::inexact_from_bounds(eta, &pb, &x).unwrap();

        // iterate through the keys and output stats about differences in magnitude
        let mut zero_round = 0;
        let mut total_diff = Float::with_val(53, 0);
        let mut total_weight = Float::with_val(53, 0);
        for k in wt.weights.keys() {
            let z = Float::with_val(53, 0);
            let true_weight = wt.weights.get(k).unwrap();
            let inex_weight = wt_inexact.weights.get(k).unwrap_or(&z);
            if inex_weight.is_zero() && !true_weight.is_zero() {
                zero_round += 1;
            }
            let diff = Float::with_val(53, true_weight - inex_weight);
            total_diff = total_diff + diff;
            total_weight = total_weight + true_weight;
            println!(
                "{:?},{:?},{:?}",
                k,
                true_weight.to_f64(),
                inex_weight.to_f64()
            );
        }
        println!("{:?} positive weights rounded to zero", zero_round);
        println!("{:?} weight lost out of {:?}", total_diff, total_weight);
    }
}
