//! Compute partition bounds for the integer partition exponential mechanism.
//! Some of the bounds strategies are due to Blocki, Datta and Bonneau '16,
//! and others are described in upcoming work.
//!
use crate::error::Fallible;
use crate::measurements::b2dp::mechanisms::laplace::clamped_laplace_mechanism;
use crate::measurements::b2dp::{Eta, ExponentialOptions};
use rug::rand::ThreadRandGen;

#[derive(Debug)]
pub struct PartitionBoundOptions {
    /// Options to pass for any executions of the exponential
    /// mechanism.
    pub exp_options: ExponentialOptions,
    /// Optional budget to use for sparsity control.
    pub sparsity_control: Option<Eta>,
    /// Padding coefficient to use for sparsity control (multiplied
    /// by 1/epsilon)
    pub sparsity_padding: f64,
    /// Padding for noisy estimates (multiplied by 1/epsilon)
    pub estimate_padding: f64,
}
impl Default for PartitionBoundOptions {
    /// Default options for generating PartitionBounds
    /// `sparsity_control`: None
    /// `exp_options`: Default (for exponential mechanism, i.e. no optimization,
    /// minimal timing channel protection, theoretical precision.)
    fn default() -> PartitionBoundOptions {
        let exp_options: ExponentialOptions = Default::default();
        PartitionBoundOptions {
            exp_options,
            sparsity_control: None,
            sparsity_padding: 1.5,
            estimate_padding: 1.5,
        }
    }
}

/// Bounds for an integer partition exponential mechanism invocation.
#[derive(Debug)]
pub struct PartitionBound {
    /// The upper bounds (terminates with 0)
    pub upper: Vec<i64>,
    /// The lower bounds (terminates with 0)
    pub lower: Vec<i64>,
    /// Total count (bound)
    pub count: usize,
    /// Number of cells (bound)
    pub cells: usize,
    /// Whether sparsity control is enforced
    pub sparsity_control: bool,
    /// The noisy estimates used, if any
    pub noisy_estimates: Option<Vec<f64>>,
}

impl PartitionBound {
    /// Checks that the partition bounds are valid:
    ///   * Upper bound always greater than or equal to lower bound
    ///   * Terminating zeros
    pub fn check(&self) -> Fallible<()> {
        if self.upper.len() != self.lower.len() {
            return fallible!(FailedFunction, "Bound length mismatch.");
        }
        for i in 0..self.upper.len() {
            if self.upper[i] < self.lower[i] {
                return fallible!(FailedFunction, "Bound inversion");
            }
        }
        if self.upper[self.upper.len() - 1] != 0 {
            return fallible!(FailedFunction, "Unterminated bound");
        }
        Ok(())
    }

    /// Get partition bounds for the given total count based on the naive
    /// formula `upper[i] <= total_count/i`, `lower[i] >= 0`.
    /// ## Arguments:
    ///   * `total_count`: upper bound on the total count
    /// ## Returns
    /// A `PartitionBound` constructed via the naive formula.
    pub fn new(total_count: usize) -> Fallible<PartitionBound> {
        let mut upper: Vec<i64> = Vec::with_capacity(total_count as usize);
        let mut lower: Vec<i64> = vec![0; total_count + 1 as usize];
        lower[0] = 1;
        for i in 0..total_count {
            let t: i64 = (total_count / (i + 1)) as i64;
            upper.push(t);
        }

        // Add terminating pair.
        upper.push(0);

        let pb = PartitionBound {
            upper,
            lower,
            count: total_count,
            cells: total_count,
            sparsity_control: false,
            noisy_estimates: None,
        };

        Ok(pb)
    }

    /// Get partition bounds for the given by the naive formula for `total_count`,
    /// with maximum length dictated by `total_cells`.
    /// ## Arguments:
    ///   * `total_count`: the total size of the partition.
    ///   * `total_cells`: (upper bound on) the total number of cells/length of the partition.
    pub fn with_cells(total_count: usize, total_cells: usize) -> Fallible<PartitionBound> {
        let mut upper: Vec<i64> = Vec::with_capacity(total_cells as usize);
        let mut lower: Vec<i64> = vec![0; total_cells + 1 as usize];
        lower[0] = 1;
        for i in 0..total_cells {
            let s: f64 = (total_count as f64) / (i as f64 + 1.0);
            let t: i64 = s.trunc() as i64;
            upper.push(t);
        }
        // Terminating pair
        upper.push(0);

        let pb = PartitionBound {
            upper,
            lower,
            count: total_count,
            cells: total_cells,
            sparsity_control: false,
            noisy_estimates: None,
        };

        return Ok(pb);
    }

    /// Computes the largest value at index `i` of any integer partition within distance `d` of `x`.
    /// Modified from Blocki, Datta and Bonneau '16.
    fn get_upper_bound(d: usize, i: usize, x: &Vec<i64>) -> i64 {
        let mut a: i64 = 2 * d as i64;

        // Make a temporary copy of the relevant range of x
        let mut t = 0;
        if (a as usize) < i {
            t = i - (a as usize);
        }

        let mut x_temp: Vec<i64> = Vec::new();
        for j in t..i + 1 {
            if j < x.len() {
                x_temp.push(x[j]);
            } else {
                x_temp.push(0);
            }
        }

        let k = x_temp.len() - 1;
        let mut j;
        while a > 0 {
            j = k;
            while
            /*j >= 0 &&*/
            x_temp[j] <= x_temp[k] && a >= 0 {
                x_temp[j] += 1;
                a -= 1;
                if j > 0 {
                    j -= 1;
                } else {
                    break;
                }
            }
        }
        if a < 0 {
            return x_temp[k] - 1;
        } else {
            return x_temp[k];
        }
    }

    /// Computes the smallest value at index `i` of any integer partition within distance `d` of `x`.
    /// Modified from Blocki, Datta and Bonneau '16.
    fn get_lower_bound(d: usize, i: usize, x: &Vec<i64>) -> i64 {
        let mut r: i64 = 2 * d as i64;

        // Make a temporary copy of the relevant range of x
        let t = (r as usize) + i + 1;
        let mut x_temp: Vec<i64> = Vec::new();
        for j in i..t {
            if j < x.len() {
                x_temp.push(x[j]);
            } else {
                x_temp.push(0);
            }
        }
        let k = 0;
        let mut j;
        while r > 0 {
            j = k;
            while j < x_temp.len() && x_temp[j] >= x_temp[k] && r >= 0 {
                x_temp[j] -= 1;
                r -= 1;
                j += 1;
            }
        }
        if r < 0 || x_temp[k] < 0 {
            return x_temp[k] + 1;
        } else {
            return x_temp[k];
        }
    }

    /// Returns partition bounds of distance at most `d` from the given partition `x`.
    ///
    /// For appropriately derived `d`, these bounds can be used to implement the
    /// approximately differentially private integer partition mechanism of
    /// Bocki, Datta and Bonneau '16.
    /// ## Arguments:
    /// * `d`: the distance bound
    /// * `x`: the partition
    /// * `total_count`: an upper bound on the total size of the partition
    /// * `total_cells`: an upper bound on the total number of cells
    pub fn from_dist(
        d: usize,
        x: &Vec<i64>,
        total_count: usize,
        total_cells: usize,
    ) -> Fallible<PartitionBound> {
        // if x is longer than total_cells, truncate it
        let y = x;
        if x.len() > total_cells {
            let mut y = vec![0; total_cells];
            y.clone_from_slice(&x[0..total_cells]);
        }

        // Initialize vectors for the bounds
        let mut upper: Vec<i64> = Vec::new();
        let mut lower: Vec<i64> = Vec::new();

        for i in 0..total_cells {
            let next_ub = PartitionBound::get_upper_bound(d, i, y);
            let next_lb = PartitionBound::get_lower_bound(d, i, y);
            upper.push(next_ub);
            lower.push(next_lb);
            if next_ub == 0 {
                break;
            }
        }
        // Add terminating pair
        if upper[upper.len() - 1] != 0 {
            upper.push(0);
            lower.push(0);
        }
        let pb = PartitionBound {
            upper: upper,
            lower: lower,
            count: total_count,
            cells: total_cells,
            sparsity_control: false,
            noisy_estimates: None,
        };
        Ok(pb)
    }

    /// Get bounds based on a reference partition using budget to compute the
    /// dist from the reference to set the bounds.
    ///
    /// TODO: not implemented;
    ///
    /// ## Arguments
    ///   * `ref_partition`: the reference partition to base the bounds on.
    ///   * `x`: the target (private) integer partition.
    ///   * `eta`: the privacy budget to compute a noisy estimate of the dist between
    ///      `ref_partition` and the target partition `x`.
    ///
    /// `x` and `r` are expected to be the same length and to have roughly the same size.
    /// ## Returns
    /// Computes a noisy estimate of the dist between `r` and `x` and returns an pair of partition
    /// bounds of length `ref_partition.len()+1` (for trailing zero pair),
    /// which are ~centered on the provided reference `r` and include all integer partitions within
    /// the estimated noisy distance.  
    pub fn with_reference(
        _total_count: usize,
        _ref: &Vec<i64>,
        _x: &Vec<i64>,
        _eta: Eta,
    ) -> Fallible<PartitionBound> {
        fallible!(FailedFunction, "Not implemented")
    }

    /// Partition bounds from noisy estimates
    ///
    /// ## Arguments
    ///   * `total_count`: upper bound on the total count
    ///   * `total_cells`: an optional upper bound on the total cells (otherwise `total_count` is used)
    ///   * `x`: the target (private) integer partition
    ///   * `eta`: the privacy budget for computing the noisy estimates
    ///   * `options`: a set of `PartitionBoundOptions` including options for exponential mechanism invocations,
    ///      padding options for noisy estimates, an optional additional privacy budget for sparsity control. If the noisy
    ///      sparsity control estimate indicates more cells than `total_cells` if provided, `total_cells`
    ///      is used.
    ///
    /// ### Example Usage
    /// ```
    /// use b2dp::{Eta, GeneratorOpenSSL, utilities::bounds::{PartitionBound, PartitionBoundOptions}};
    /// let mut rng = GeneratorOpenDP::default();
    /// let eta = Eta::new(1,1,1).unwrap();
    /// let x: Vec<i64> = vec![5,4,3,2,1,0];
    /// let total_count = 15;
    /// let total_cells = None;
    /// let options: PartitionBoundOptions = Default::default();
    /// let pb = PartitionBound::from_noisy_estimates(total_count,
    ///                                               total_cells,
    ///                                               &x,
    ///                                               eta,
    ///                                               rng,
    ///                                               options);
    /// ```
    pub fn from_noisy_estimates<R: ThreadRandGen>(
        total_count: usize,
        total_cells: Option<usize>,
        x: &Vec<i64>,
        eta: Eta,
        rng: &mut R,
        options: PartitionBoundOptions,
    ) -> Fallible<PartitionBound> {
        let mut cells = total_count;
        // Check that total_cells is valid
        if total_cells.is_some() {
            cells = total_cells.unwrap();
        }
        if cells == 0 {
            return fallible!(FailedFunction, "Cell count must be positive.");
        }

        // If sparsity control is specified, get an estimate of the number of zeros
        // and adjust cells to the noisy number of non-zero cells plus padding
        // proportional to the standard deviation.
        if let Some(sparsity_budget) = options.sparsity_control {
            // Get noisy estimate of number of non-zero elements
            let non_zeros = x.iter().filter(|&y| *y > 0).count();
            let mut noisy_non_zeros = clamped_laplace_mechanism(
                sparsity_budget,
                0.0,
                cells as f64,
                non_zeros as f64,
                1.0,
                rng,
                options.exp_options,
            )
            .unwrap();

            // Add standard deviation padding
            let eps = sparsity_budget.get_approximate_epsilon();
            let stdev = options.sparsity_padding * (1.0 / eps);
            noisy_non_zeros += stdev;

            // Round to an integer and convert to usize
            let noisy_non_zeros: usize = noisy_non_zeros as usize;

            // Only update number of cells if it's smaller than what we know
            if noisy_non_zeros < cells {
                cells = noisy_non_zeros;
            }
        }

        // Get the bounds dicated by the cell count.
        let pb_baseline = PartitionBound::with_cells(total_count, cells).unwrap();

        // Initialize vectors for the bounds
        let mut upper_bound: Vec<i64> = Vec::new();
        let mut lower_bound: Vec<i64> = Vec::new();
        let mut noisy_estimates: Vec<f64> = Vec::new();
        for i in 0..cells {
            let mut xi = 0;
            if i < x.len() {
                xi = x[i];
            }

            // For each index, draw a noisy value
            // Initial version: just draw between 0, total_count
            let gamma = 1.0;
            let noisy_estimate = clamped_laplace_mechanism(
                eta,
                0.0,
                total_count as f64,
                xi as f64,
                gamma,
                rng,
                options.exp_options,
            )
            .unwrap();
            noisy_estimates.push(noisy_estimate);
            let noisy_estimate: i64 = noisy_estimate as i64;

            // Add standard deviation padding
            let eps = eta.get_approximate_epsilon();
            let stdev = (options.estimate_padding * (1.0 / eps)) as i64;

            // Compute upper and lower bounds based on the noisy value and stdev
            // padding
            let upper = noisy_estimate + stdev;
            let lower = noisy_estimate - stdev;

            // Add to the bounds
            upper_bound.push(upper);
            lower_bound.push(lower);
        }

        // Sort the bounds in descending order
        upper_bound.sort();
        upper_bound.reverse();
        lower_bound.sort();
        lower_bound.reverse();

        for i in 0..upper_bound.len() {
            // Modify the bounds based on feasible settings (e.g., must be within the
            // bounds dictated by the baseline
            let mut lower = lower_bound[i];
            let mut upper = upper_bound[i];
            if lower < pb_baseline.lower[i] {
                lower = pb_baseline.lower[i];
            }
            if lower > pb_baseline.upper[i] {
                lower = pb_baseline.upper[i];
            }
            if upper > pb_baseline.upper[i] {
                upper = pb_baseline.upper[i];
            }
            if upper < pb_baseline.lower[i] {
                upper = pb_baseline.lower[i];
            }
            if upper < lower {
                return fallible!(FailedFunction, "Bounds out of order.");
            }
            upper_bound[i] = upper;
            lower_bound[i] = lower;
        }

        // Append trailing zero pair
        upper_bound.push(0);
        lower_bound.push(0);

        let pb = PartitionBound {
            upper: upper_bound,
            lower: lower_bound,
            count: total_count,
            cells: cells,
            sparsity_control: options.sparsity_control.is_some(),
            noisy_estimates: Some(noisy_estimates),
        };
        Ok(pb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::samplers::GeneratorOpenDP;
    #[test]
    fn test_from_dist() {
        for total_cells in 5..10 {
            let x: Vec<i64> = vec![5, 4, 3, 2, 1, 0];
            let total_count: i64 = x.iter().sum();
            let d = 2;
            let pb = PartitionBound::from_dist(d, &x, total_count as usize, total_cells).unwrap();

            assert!(pb.check().is_ok());
            // Check dist constraints
            for i in 0..x.len() {
                assert!(pb.upper[i] - x[i] <= 2 * d as i64);
                assert!(x[i] - pb.lower[i] <= 2 * d as i64);
                assert!(pb.upper[i] >= x[i]);
                assert!(pb.lower[i] <= x[i]);
            }
        }
    }

    #[test]
    fn test_from_noisy_estimates() {
        let x: Vec<i64> = vec![5, 4, 3, 2, 1, 0];
        let eta = Eta::new(1, 1, 1).unwrap();
        let total_cells = 6;
        let total_count: i64 = x.iter().sum();
        let mut rng = GeneratorOpenDP::default();
        let options: PartitionBoundOptions = Default::default();
        let pb = PartitionBound::from_noisy_estimates(
            total_count as usize,
            Some(total_cells),
            &x,
            eta,
            &mut rng,
            options,
        )
        .unwrap();
        let baseline_pb = PartitionBound::with_cells(total_count as usize, total_cells).unwrap();
        assert_eq!(pb.upper.len(), baseline_pb.upper.len());
        for i in 0..pb.upper.len() {
            assert!(pb.upper[i] <= baseline_pb.upper[i]);
            assert!(pb.lower[i] >= baseline_pb.lower[i]);
            assert!(pb.upper[i] >= pb.lower[i]);
        }
    }

    #[test]
    fn test_from_noisy_estimates_with_sparsity() {
        let x: Vec<i64> = vec![5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let eta = Eta::new(1, 1, 1).unwrap();
        let sparsity_eta = Eta::new(1, 1, 1).unwrap();
        let total_cells = 10;
        let total_count: i64 = x.iter().sum();
        let mut rng = GeneratorOpenDP::default();
        let options = PartitionBoundOptions {
            sparsity_control: Some(sparsity_eta),
            ..Default::default()
        };
        let pb = PartitionBound::from_noisy_estimates(
            total_count as usize,
            Some(total_cells),
            &x,
            eta,
            &mut rng,
            options,
        )
        .unwrap();
        let baseline_pb = PartitionBound::with_cells(total_count as usize, total_cells).unwrap();
        assert!(pb.upper.len() <= baseline_pb.upper.len());
        for i in 0..pb.upper.len() {
            assert!(pb.upper[i] <= baseline_pb.upper[i]);
            assert!(pb.lower[i] >= baseline_pb.lower[i]);
            assert!(pb.upper[i] >= pb.lower[i]);
        }
    }
    #[test]
    fn test_basic_bounds() {
        let pb = PartitionBound::new(5).unwrap();
        let ub: Vec<i64> = vec![5, 2, 1, 1, 1];
        let lb: Vec<i64> = vec![1, 0, 0, 0, 0];
        for i in 0..ub.len() {
            assert_eq!(ub[i], pb.upper[i]);
            assert_eq!(lb[i], pb.lower[i]);
        }
        assert_eq!(pb.upper[ub.len()], 0);
        assert_eq!(pb.lower[ub.len()], 0);
    }

    #[test]
    fn test_with_cells() {
        let pb = PartitionBound::with_cells(5, 6).unwrap();
        let ub: Vec<i64> = vec![5, 2, 1, 1, 1, 0];
        let lb: Vec<i64> = vec![1, 0, 0, 0, 0, 0];

        for i in 0..ub.len() {
            assert_eq!(ub[i], pb.upper[i]);
            assert_eq!(lb[i], pb.lower[i]);
        }
        assert_eq!(pb.upper[ub.len()], 0);
        assert_eq!(pb.lower[ub.len()], 0);
    }
}
