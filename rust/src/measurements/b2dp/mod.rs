/// # Base-2 Differential Privacy
/// Implements the exponential mechanism and other utilities for base-2 
/// Differential Privacy, based on [Ilvento '19](https://arxiv.org/abs/1912.04222).
/// 
/// **Status:** active development, reference implementation only. Not 
/// intended for uses other than research. **Subject to change without notice.**
/// 
/// ## Background
/// Although the exponential mechanism does not directly reveal the result of inexact
/// floating point computations, it has been shown to be vulnerable to attacks based
/// on rounding and no-op addition behavior of floating point arithmetic. To prevent
/// these issues, base-2 differential privacy uses arithmetic with base 2, rather than 
/// base e, allowing for an exact implementation. This crate implements the base-2 exponential
/// mechanism, (experimental) sparse vector and (experimental) integer partitions, as 
/// well as (experimental) noisy threshold and (experimental) clamped Laplace. It also 
/// includes useful base-2 DP utilities for parameter conversion.
/// 
/// This code is under active development and should be treated as a reference
/// for research purposes only (particularly anything marked *experimental*). 
/// 
/// ## Mechanism Details
/// * Base-2 exponential mechanism and parameter construction are described in
///   this [paper](https://arxiv.org/abs/1912.04222).
/// * The integer partition exponential mechanism is based on extensions of
///   the mechanism proposed by Blocki, Datta and Bonneau in this [paper](http://www.jbonneau.com/doc/BDB16-NDSS-pw_list_differential_privacy.pdf).
///   Extensions include a pure-DP version of the mechanism and bias computation, 
///   and are described in this [working paper](https://github.com/cilvento/b2dp/blob/master/docs/working_papers/integer_partitions.pdf).
/// * The sparse vector mechanism implementation is based on a [working paper](https://github.com/cilvento/b2dp/blob/master/docs/working_papers/sparse_vector.pdf) that describes
///   the dangers of inexact implementation of sparse vector, and in particular
///   how randomness alignment must be adjusted to deal with finite values. 
/// 
/// ## Example Usage
/// **Converting a base-e parameter to base-2**
/// ```
/// use b2dp::Eta;
/// # use b2dp::errors::*;
/// # fn main() -> Result<()> {
/// let epsilon = 1.25;
/// let eta = Eta::from_epsilon(epsilon)?;
/// # Ok(()) }
/// ```
/// **Running the exponential mechanism**
/// 
/// Run the exponential mechanism with utility function `utility_fn`.
/// The utility function is negated by convention, and utilities must
/// be non-negative values. For example, using utility range `0` to `10`,
/// utility `0` has the highest weight and probability of selection and 
/// utility `10` the lowest. 
/// 
/// ```
/// use b2dp::{exponential_mechanism, Eta, GeneratorOpenSSL, errors::*};
/// 
/// # fn main() -> Result<()> {
/// fn util_fn (x: &u32) -> f64 {
///     return ((*x as f64)-0.0).abs();
/// }
/// let eta = Eta::new(1,1,1)?; // Construct a privacy parameter
/// let utility_min = 0; // Set bounds on the utility and outcomes
/// let utility_max = 10;
/// let max_outcomes = 10;
/// let mut rng = GeneratorOpenDP::default();
/// let outcomes: Vec<u32> = (0..max_outcomes).collect();
/// let sample = exponential_mechanism(eta, &outcomes, util_fn, 
///                                     utility_min, utility_max, 
///                                     max_outcomes,
///                                     rng, 
///                                     Default::default())?;
/// # Ok(()) 
/// # }
/// ```
/// **Scaling based on utility function sensitivity**
/// Given a utility function with sensitivity `alpha`, the `exponential_mechanism` 
/// implementation is `2*alpha*ln(2)*eta` base-e DP. To explicitly scale by `alpha`
/// the caller can either modify the `eta` used or the utility function.
/// ```
/// use b2dp::{exponential_mechanism, Eta, GeneratorOpenSSL, errors::*};
/// # fn main() -> Result<()> {
/// // Scale the privacy parameter to account for the utility sensitivity
/// let epsilon = 1.25;
/// let eta = Eta::from_epsilon(epsilon)?;
/// let alpha = 2.0;
/// let eta_scaled = Eta::from_epsilon(epsilon/alpha)?;
/// // Or scale the utility function to reduce sensitivity
/// let alpha = 2.0;
/// 
/// fn util_fn (x: &u32) -> f64 {
///     return (2.0*(*x as f64)-0.0).abs();
/// }
/// let scaled_utility_fn = |x: &f64| -> f64 { *x/alpha };
/// # Ok(())
/// # }
/// ```
/// **Sparse Vector** an exact implementation of discrete sparse vector. 
/// Takes in a set of query values (does not currently support a query function 
/// interface) and returns `true` or `false` depending on whether each query 
/// exceeds the fixed threshold of `0`. 
/// 
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL, sparse_vector, errors::*};
/// # use rug::Float;
/// # fn main() -> Result<()> {
/// let eta1 = Eta::new(1,1,2)?;
/// let eta2 = Eta::new(1,1,2)?;
/// let c = 2;
/// let queries = vec![1.0,2.0,3.0,4.0,5.0,1.0];
/// let gamma = 0.5;
/// let q_min = 0.0;
/// let q_max = 6.0;
/// let w = 5.0;
/// let mut rng = GeneratorOpenDP::default();
/// let optimize = false;
/// let outputs = sparse_vector(eta1, eta2, c, &queries, gamma, q_min, q_max, w, &mut rng, optimize)?;
/// # Ok(())
/// # }
/// ```
/// 
/// **Sparse Vector *with gap*** an exact implementation of discrete 
/// sparse vector. Takes in a set of query values (does not 
/// currently support a query function interface) and gaps and returns the 
/// largest gap if the noisy query exceeds the fixed noisy threshold of 0.
/// 
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL, sparse_vector_with_gap, errors::*};
/// # use rug::Float;
/// # fn main() -> Result<()> {
/// let eta1 = Eta::new(1,1,2)?;
/// let eta2 = Eta::new(1,1,2)?;
/// let c = 2;
/// let queries = vec![1.0,2.0,3.0,4.0,5.0,1.0];
/// let gaps = vec![1.0, 2.0, 3.0];
/// let gamma = 0.5;
/// let q_min = 0.0;
/// let q_max = 6.0;
/// let w = 5.0;
/// let mut rng = GeneratorOpenDP::default();
/// let optimize = false;
/// let outputs = sparse_vector_with_gap(eta1, eta2, c, &gaps, &queries, gamma, q_min, q_max, w, &mut rng, optimize)?;
/// # Ok(())
/// # }
/// ```
/// 
/// **Lazy Threshold** [`lazy_threshold`](./utilities/discretesampling/fn.lazy_threshold.html) determines whether discrete Laplace noise
/// centered at `0` with granularity `gamma` exceeds the given `threshold`. 
/// 
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,utilities::exactarithmetic::ArithmeticConfig, lazy_threshold, errors::*};
/// # use rug::Float;
/// # fn main() -> Result<()> {
/// let eta = Eta::new(1,1,2)?; // can be adjusted for the desired value of gamma.
/// let mut arithmeticconfig = ArithmeticConfig::basic()?;
/// let mut rng = GeneratorOpenDP::default();
/// let gamma_inv = Float::with_val(arithmeticconfig.precision, 2);
/// let threshold = Float::with_val(arithmeticconfig.precision, 0);
/// arithmeticconfig.enter_exact_scope()?; 
/// let s = lazy_threshold(eta, & mut arithmeticconfig, &gamma_inv, &threshold, &mut rng, false)?;
/// assert!(!s.is_finite()); // returns plus or minus infinity
/// if s.is_sign_positive() { /* Greater than the threshold */ ;}
/// else { /* Less than the threshold. */ ;}
/// let b = arithmeticconfig.exit_exact_scope();
/// assert!(b.is_ok()); // Must check that no inexact arithmetic was performed. 
/// # Ok(())
/// # }
/// ```
/// 
/// **Sample within Bounds**: samples from the Discrete Laplace mechanisms within the bounds,
/// where boundary values are sampled with sum of probabilities of all values less than (or greater than)
/// the bound.
/// 
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,utilities::exactarithmetic::ArithmeticConfig, sample_within_bounds, errors::*};
/// # use rug::Float;
/// # fn main() -> Result<()> {
/// # let eta = Eta::new(1,1,2)?; // construct eta that can be adjusted for the desired value of gamma.
/// # let mut arithmeticconfig = ArithmeticConfig::basic()?;
/// # let mut rng = GeneratorOpenDP::default();
/// let gamma = Float::with_val(arithmeticconfig.precision, 0.5);
/// let wmin = Float::with_val(arithmeticconfig.precision, -5);
/// let wmax = Float::with_val(arithmeticconfig.precision, 5);
/// arithmeticconfig.enter_exact_scope()?;
/// let s = sample_within_bounds(eta, &gamma, &wmin, &wmax, & mut arithmeticconfig, &mut rng,false)?;
/// let b = arithmeticconfig.exit_exact_scope();
/// assert!(b.is_ok()); // Must check that no inexact arithmetic was performed. 
/// # Ok(())
/// # }
/// ```
/// 
/// **Integer Partitions**: a sample invocation given a distance `d` for the 
/// integer partition exponential mechanism as in Blocki, Datta and Bonneau '16.
/// ```
/// # use b2dp::{Eta,GeneratorOpenSSL,integer_partition_mechanism_with_bounds, PartitionBound, errors::*};
/// # use rug::Float;
/// # fn main() -> Result<()> {
/// let eta = Eta::new(1,1,1)?;
/// let d = 5;
/// let x: Vec<i64> = vec![5,4,3,2,1,0];
/// let total_count = 15; // upper bound on total count
/// let total_cells = x.len() + d;
/// let pb = PartitionBound::from_dist(d, &x, total_count, total_cells)?;
/// let y = integer_partition_mechanism_with_bounds(eta, &x, &pb, Default::default())?;
/// # Ok(())
/// # }
/// ```

/// Base-2 Differential Privacy Utilities
pub mod utilities;
/// Base-2 Differential Privacy Mechanisms
pub mod mechanisms;

// Parameters and main exponential mechanism functionality
pub use utilities::params::Eta as Eta;
pub use utilities::exactarithmetic::randomized_round;
pub use utilities::exactarithmetic::normalized_sample;
pub use mechanisms::exponential::exponential_mechanism;
pub use mechanisms::exponential::ExponentialOptions;

// Integer Partitions
pub use mechanisms::integerpartition::integer_partition_mechanism_with_bounds;
pub use mechanisms::integerpartition::IntegerPartitionOptions;
pub use utilities::bounds::{PartitionBound,PartitionBoundOptions};

// Discrete Laplace
pub use utilities::discretesampling::lazy_threshold;
pub use utilities::discretesampling::conditional_lazy_threshold;
pub use utilities::discretesampling::sample_within_bounds;

// Sparse Vector
pub use mechanisms::sparsevector::sparse_vector;
pub use mechanisms::sparsevector::sparse_vector_with_gap;


