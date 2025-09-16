# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' alp queryable constructor
#'
#' Measurement to release a queryable containing a DP projection of bounded sparse data.
#'
#' The size of the projection is O(total * size_factor * scale / alpha).
#' The evaluation time of post-processing is O(beta * scale / alpha).
#'
#' `size_factor` is an optional multiplier (defaults to 50) for setting the size of the projection.
#' There is a memory/utility trade-off.
#' The value should be sufficiently large to limit hash collisions.
#'
#'
#' Required features: `contrib`
#'
#' [make_alp_queryable in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_alp_queryable.html)
#'
#' **Citations:**
#'
#' * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `MapDomain<AtomDomain<K>, AtomDomain<CI>>`
#' * Output Type:    `L01InfDistance<AbsoluteDistance<CI>>`
#' * Input Metric:   `MaxDivergence`
#' * Output Measure: `Queryable<K, f64>`
#'
#' @concept measurements
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param scale Privacy loss parameter. This is equal to epsilon/sensitivity.
#' @param total_limit Either the true value or an upper bound estimate of the sum of all values in the input.
#' @param value_limit Upper bound on individual values (referred to as β). Entries above β are clamped.
#' @param size_factor Optional multiplier (default of 50) for setting the size of the projection.
#' @param alpha Optional parameter (default of 4) for scaling and determining p in randomized response step.
#' @return Measurement
#' @export
make_alp_queryable <- function(
  input_domain,
  input_metric,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L
) {
  assert_features("contrib")

  # Standardize type arguments.
  .CI <- get_value_type(get_carrier_type(input_domain))
  .T.value_limit <- new_runtime_type(origin = "Option", args = list(.CI))
  .T.size_factor <- new_runtime_type(origin = "Option", args = list(u32))
  .T.alpha <- new_runtime_type(origin = "Option", args = list(u32))

  log_ <- new_constructor_log("make_alp_queryable", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "total_limit", "value_limit", "size_factor", "alpha"),
    list(input_domain, input_metric, unbox2(scale), total_limit, value_limit, size_factor, alpha)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .CI, inferred = rt_infer(total_limit))
  rt_assert_is_similar(expected = .T.value_limit, inferred = rt_infer(value_limit))
  rt_assert_is_similar(expected = .T.size_factor, inferred = rt_infer(size_factor))
  rt_assert_is_similar(expected = .T.alpha, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_alp_queryable",
    input_domain, input_metric, scale, total_limit, value_limit, size_factor, alpha, .CI, rt_parse(.T.value_limit), rt_parse(.T.size_factor), rt_parse(.T.alpha),
    log_, PACKAGE = "opendp")
  output
}

#' partial alp queryable constructor
#'
#' See documentation for [make_alp_queryable()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Privacy loss parameter. This is equal to epsilon/sensitivity.
#' @param total_limit Either the true value or an upper bound estimate of the sum of all values in the input.
#' @param value_limit Upper bound on individual values (referred to as β). Entries above β are clamped.
#' @param size_factor Optional multiplier (default of 50) for setting the size of the projection.
#' @param alpha Optional parameter (default of 4) for scaling and determining p in randomized response step.
#' @return Measurement
#' @export
then_alp_queryable <- function(
  lhs,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L
) {

  log_ <- new_constructor_log("then_alp_queryable", "measurements", new_hashtab(
    list("scale", "total_limit", "value_limit", "size_factor", "alpha"),
    list(unbox2(scale), total_limit, value_limit, size_factor, alpha)
  ))

  make_chain_dyn(
    make_alp_queryable(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      total_limit = total_limit,
      value_limit = value_limit,
      size_factor = size_factor,
      alpha = alpha),
    lhs,
    log_)
}


#' canonical noise constructor
#'
#' Make a Measurement that adds noise from a canonical noise distribution.
#' The implementation is tailored towards approximate-DP,
#' resulting in noise sampled from the Tulap distribution.
#'
#'
#' Required features: `contrib`
#'
#' [make_canonical_noise in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_canonical_noise.html)
#'
#' **Citations:**
#'
#' - [AV23 Canonical Noise Distributions and Private Hypothesis Tests](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Canonical-noise-distributions-and-private-hypothesis-tests/10.1214/23-AOS2259.short)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<f64>`
#' * Output Type:    `AbsoluteDistance<f64>`
#' * Input Metric:   `Approximate<MaxDivergence>`
#' * Output Measure: `f64`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/canonical_noise/make_canonical_noise.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric of the input.
#' @param d_in Sensitivity
#' @param d_out Privacy parameters (ε, δ)
#' @return Measurement
#' @export
make_canonical_noise <- function(
  input_domain,
  input_metric,
  d_in,
  d_out
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T.d_out <- new_runtime_type(origin = "Tuple", args = list(f64, f64))

  log_ <- new_constructor_log("make_canonical_noise", "measurements", new_hashtab(
    list("input_domain", "input_metric", "d_in", "d_out"),
    list(input_domain, input_metric, unbox2(d_in), lapply(d_out, unbox2))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(d_in))
  rt_assert_is_similar(expected = .T.d_out, inferred = rt_infer(d_out))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_canonical_noise",
    input_domain, input_metric, d_in, d_out, rt_parse(.T.d_out),
    log_, PACKAGE = "opendp")
  output
}

#' partial canonical noise constructor
#'
#' See documentation for [make_canonical_noise()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param d_in Sensitivity
#' @param d_out Privacy parameters (ε, δ)
#' @return Measurement
#' @export
then_canonical_noise <- function(
  lhs,
  d_in,
  d_out
) {

  log_ <- new_constructor_log("then_canonical_noise", "measurements", new_hashtab(
    list("d_in", "d_out"),
    list(unbox2(d_in), lapply(d_out, unbox2))
  ))

  make_chain_dyn(
    make_canonical_noise(
      output_domain(lhs),
      output_metric(lhs),
      d_in = d_in,
      d_out = d_out),
    lhs,
    log_)
}


#' gaussian constructor
#'
#' Make a Measurement that adds noise from the Gaussian(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`          |
#' | ------------------------------- | ------------ | ----------------------- |
#' | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
#'
#'
#' Required features: `contrib`
#'
#' [make_gaussian in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_gaussian.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noise/distribution/gaussian/make_gaussian.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
#' @return Measurement
#' @examples
#' library(opendp)
#' enable_features("contrib")
#' gaussian <- make_gaussian(
#'   atom_domain(.T = f64, nan = FALSE),
#'   absolute_distance(.T = f64),
#'   scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Or, more readably, define the space and then chain:
#' space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Sensitivity of this measurement:
#' gaussian(d_in = 1)
#' gaussian(d_in = 2)
#' gaussian(d_in = 4)
#'
#' # Typically will be used with vectors rather than individual numbers:
#' space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = i32))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = c(10L, 20L, 30L))
#' @export
make_gaussian <- function(
  input_domain,
  input_metric,
  scale,
  k = NULL,
  .MO = "ZeroConcentratedDivergence"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_gaussian", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "k", "MO"),
    list(input_domain, input_metric, unbox2(scale), k, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_gaussian",
    input_domain, input_metric, scale, k, .MO, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial gaussian constructor
#'
#' See documentation for [make_gaussian()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence`.
#' @return Measurement
#' @examples
#' library(opendp)
#' enable_features("contrib")
#' gaussian <- make_gaussian(
#'   atom_domain(.T = f64, nan = FALSE),
#'   absolute_distance(.T = f64),
#'   scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Or, more readably, define the space and then chain:
#' space <- c(atom_domain(.T = f64, nan = FALSE), absolute_distance(.T = f64))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Sensitivity of this measurement:
#' gaussian(d_in = 1)
#' gaussian(d_in = 2)
#' gaussian(d_in = 4)
#'
#' # Typically will be used with vectors rather than individual numbers:
#' space <- c(vector_domain(atom_domain(.T = i32)), l2_distance(.T = i32))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = c(10L, 20L, 30L))
#' @export
then_gaussian <- function(
  lhs,
  scale,
  k = NULL,
  .MO = "ZeroConcentratedDivergence"
) {

  log_ <- new_constructor_log("then_gaussian", "measurements", new_hashtab(
    list("scale", "k", "MO"),
    list(unbox2(scale), k, .MO)
  ))

  make_chain_dyn(
    make_gaussian(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      k = k,
      .MO = .MO),
    lhs,
    log_)
}


#' gaussian threshold constructor
#'
#' Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#'
#' Required features: `contrib`
#'
#' [make_gaussian_threshold in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_gaussian_threshold.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noise_threshold/distribution/gaussian/make_gaussian_threshold.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric for the input domain.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude pairs with values whose distance from zero exceeds this value.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure.
#' @return Measurement
#' @export
make_gaussian_threshold <- function(
  input_domain,
  input_metric,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<ZeroConcentratedDivergence>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO, generics = list(".TV"))
  .TV <- get_value_type(get_carrier_type(input_domain))
  .MO <- rt_substitute(.MO, .TV = .TV)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_gaussian_threshold", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "threshold", "k", "MO"),
    list(input_domain, input_metric, unbox2(scale), threshold, k, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .TV, inferred = rt_infer(threshold))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_gaussian_threshold",
    input_domain, input_metric, scale, threshold, k, .MO, .TV, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial gaussian threshold constructor
#'
#' See documentation for [make_gaussian_threshold()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude pairs with values whose distance from zero exceeds this value.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure.
#' @return Measurement
#' @export
then_gaussian_threshold <- function(
  lhs,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<ZeroConcentratedDivergence>"
) {

  log_ <- new_constructor_log("then_gaussian_threshold", "measurements", new_hashtab(
    list("scale", "threshold", "k", "MO"),
    list(unbox2(scale), threshold, k, .MO)
  ))

  make_chain_dyn(
    make_gaussian_threshold(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      threshold = threshold,
      k = k,
      .MO = .MO),
    lhs,
    log_)
}


#' geometric constructor
#'
#' Equivalent to `make_laplace` but restricted to an integer support.
#' Can specify `bounds` to run the algorithm in near constant-time.
#'
#'
#' Required features: `contrib`
#'
#' [make_geometric in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_geometric.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noise/distribution/geometric/make_geometric.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
#' @param bounds Set bounds on the count to make the algorithm run in constant-time.
#' @param .MO Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
#' @return Measurement
#' @export
make_geometric <- function(
  input_domain,
  input_metric,
  scale,
  bounds = NULL,
  .MO = "MaxDivergence"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO, generics = list(".T", ".OptionT"))
  .T <- get_atom(get_carrier_type(input_domain))
  .OptionT <- new_runtime_type(origin = "Option", args = list(new_runtime_type(origin = "Tuple", args = list(.T, .T))))
  .MO <- rt_substitute(.MO, .T = .T, .OptionT = .OptionT)

  log_ <- new_constructor_log("make_geometric", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "bounds", "MO"),
    list(input_domain, input_metric, unbox2(scale), bounds, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .OptionT, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_geometric",
    input_domain, input_metric, scale, bounds, .MO, .T, .OptionT,
    log_, PACKAGE = "opendp")
  output
}

#' partial geometric constructor
#'
#' See documentation for [make_geometric()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
#' @param bounds Set bounds on the count to make the algorithm run in constant-time.
#' @param .MO Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
#' @return Measurement
#' @export
then_geometric <- function(
  lhs,
  scale,
  bounds = NULL,
  .MO = "MaxDivergence"
) {

  log_ <- new_constructor_log("then_geometric", "measurements", new_hashtab(
    list("scale", "bounds", "MO"),
    list(unbox2(scale), bounds, .MO)
  ))

  make_chain_dyn(
    make_geometric(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      bounds = bounds,
      .MO = .MO),
    lhs,
    log_)
}


#' laplace constructor
#'
#' Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' Internally, all sampling is done using the discrete Laplace distribution.
#'
#'
#' Required features: `contrib`
#'
#' [make_laplace in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_laplace.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#' * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noise/distribution/laplace/make_laplace.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param k The noise granularity in terms of 2^k, only valid for domains over floats.
#' @param .MO Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
#' @return Measurement
#' @export
make_laplace <- function(
  input_domain,
  input_metric,
  scale,
  k = NULL,
  .MO = "MaxDivergence"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_laplace", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "k", "MO"),
    list(input_domain, input_metric, unbox2(scale), k, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_laplace",
    input_domain, input_metric, scale, k, .MO, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial laplace constructor
#'
#' See documentation for [make_laplace()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param k The noise granularity in terms of 2^k, only valid for domains over floats.
#' @param .MO Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
#' @return Measurement
#' @export
then_laplace <- function(
  lhs,
  scale,
  k = NULL,
  .MO = "MaxDivergence"
) {

  log_ <- new_constructor_log("then_laplace", "measurements", new_hashtab(
    list("scale", "k", "MO"),
    list(unbox2(scale), k, .MO)
  ))

  make_chain_dyn(
    make_laplace(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      k = k,
      .MO = .MO),
    lhs,
    log_)
}


#' laplace threshold constructor
#'
#' Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#'
#' Required features: `contrib`
#'
#' [make_laplace_threshold in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_laplace_threshold.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noise_threshold/distribution/laplace/make_laplace_threshold.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric for the input domain.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure.
#' @return Measurement
#' @export
make_laplace_threshold <- function(
  input_domain,
  input_metric,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<MaxDivergence>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO, generics = list(".TV"))
  .TV <- get_value_type(get_carrier_type(input_domain))
  .MO <- rt_substitute(.MO, .TV = .TV)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_laplace_threshold", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "threshold", "k", "MO"),
    list(input_domain, input_metric, unbox2(scale), threshold, k, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .TV, inferred = rt_infer(threshold))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_laplace_threshold",
    input_domain, input_metric, scale, threshold, k, .MO, .TV, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial laplace threshold constructor
#'
#' See documentation for [make_laplace_threshold()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure.
#' @return Measurement
#' @export
then_laplace_threshold <- function(
  lhs,
  scale,
  threshold,
  k = NULL,
  .MO = "Approximate<MaxDivergence>"
) {

  log_ <- new_constructor_log("then_laplace_threshold", "measurements", new_hashtab(
    list("scale", "threshold", "k", "MO"),
    list(unbox2(scale), threshold, k, .MO)
  ))

  make_chain_dyn(
    make_laplace_threshold(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      threshold = threshold,
      k = k,
      .MO = .MO),
    lhs,
    log_)
}


#' noise constructor
#'
#' Make a Measurement that adds noise from the appropriate distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`          |
#' | ------------------------------- | ------------ | ----------------------- |
#' | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
#'
#'
#' Required features: `contrib`
#'
#' [make_noise in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_noise.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `DI::Carrier`
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param output_measure Privacy measure. Either `MaxDivergence` or `ZeroConcentratedDivergence`.
#' @param scale Noise scale parameter.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
make_noise <- function(
  input_domain,
  input_metric,
  output_measure,
  scale,
  k = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_noise", "measurements", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "scale", "k"),
    list(input_domain, input_metric, output_measure, unbox2(scale), k)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_noise",
    input_domain, input_metric, output_measure, scale, k, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial noise constructor
#'
#' See documentation for [make_noise()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param output_measure Privacy measure. Either `MaxDivergence` or `ZeroConcentratedDivergence`.
#' @param scale Noise scale parameter.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
then_noise <- function(
  lhs,
  output_measure,
  scale,
  k = NULL
) {

  log_ <- new_constructor_log("then_noise", "measurements", new_hashtab(
    list("output_measure", "scale", "k"),
    list(output_measure, unbox2(scale), k)
  ))

  make_chain_dyn(
    make_noise(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      scale = scale,
      k = k),
    lhs,
    log_)
}


#' noise threshold constructor
#'
#' Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#'
#' Required features: `contrib`
#'
#' [make_noise_threshold in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_noise_threshold.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `MI`
#' * Input Metric:   `Approximate<MO>`
#' * Output Measure: `DI::Carrier`
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric for the input domain.
#' @param output_measure Privacy measure. Either `MaxDivergence` or `ZeroConcentratedDivergence`.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
make_noise_threshold <- function(
  input_domain,
  input_metric,
  output_measure,
  scale,
  threshold,
  k = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TV <- get_value_type(get_carrier_type(input_domain))
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log_ <- new_constructor_log("make_noise_threshold", "measurements", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "scale", "threshold", "k"),
    list(input_domain, input_metric, output_measure, unbox2(scale), threshold, k)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .TV, inferred = rt_infer(threshold))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_noise_threshold",
    input_domain, input_metric, output_measure, scale, threshold, k, .TV, rt_parse(.T.k),
    log_, PACKAGE = "opendp")
  output
}

#' partial noise threshold constructor
#'
#' See documentation for [make_noise_threshold()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param output_measure Privacy measure. Either `MaxDivergence` or `ZeroConcentratedDivergence`.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
then_noise_threshold <- function(
  lhs,
  output_measure,
  scale,
  threshold,
  k = NULL
) {

  log_ <- new_constructor_log("then_noise_threshold", "measurements", new_hashtab(
    list("output_measure", "scale", "threshold", "k"),
    list(output_measure, unbox2(scale), threshold, k)
  ))

  make_chain_dyn(
    make_noise_threshold(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      scale = scale,
      threshold = threshold,
      k = k),
    lhs,
    log_)
}


#' noisy max constructor
#'
#' Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
#'
#'
#' Required features: `contrib`
#'
#' [make_noisy_max in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_noisy_max.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Type:    `LInfDistance<TIA>`
#' * Input Metric:   `MO`
#' * Output Measure: `usize`
#'
#' @concept measurements
#' @param input_domain Domain of the input vector. Must be a non-nullable `VectorDomain`
#' @param input_metric Metric on the input domain. Must be `LInfDistance`
#' @param output_measure One of `MaxDivergence`, `ZeroConcentratedDivergence`
#' @param scale Scale for the noise distribution
#' @param negate Set to true to return min
#' @return Measurement
#' @export
make_noisy_max <- function(
  input_domain,
  input_metric,
  output_measure,
  scale,
  negate = FALSE
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_noisy_max", "measurements", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "scale", "negate"),
    list(input_domain, input_metric, output_measure, unbox2(scale), unbox2(negate))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(negate))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_noisy_max",
    input_domain, input_metric, output_measure, scale, negate,
    log_, PACKAGE = "opendp")
  output
}

#' partial noisy max constructor
#'
#' See documentation for [make_noisy_max()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param output_measure One of `MaxDivergence`, `ZeroConcentratedDivergence`
#' @param scale Scale for the noise distribution
#' @param negate Set to true to return min
#' @return Measurement
#' @export
then_noisy_max <- function(
  lhs,
  output_measure,
  scale,
  negate = FALSE
) {

  log_ <- new_constructor_log("then_noisy_max", "measurements", new_hashtab(
    list("output_measure", "scale", "negate"),
    list(output_measure, unbox2(scale), unbox2(negate))
  ))

  make_chain_dyn(
    make_noisy_max(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      scale = scale,
      negate = negate),
    lhs,
    log_)
}


#' noisy top k constructor
#'
#' Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
#'
#'
#' Required features: `contrib`
#'
#' [make_noisy_top_k in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_noisy_top_k.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Type:    `LInfDistance<TIA>`
#' * Input Metric:   `MO`
#' * Output Measure: `Vec<usize>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/noisy_top_k/make_noisy_top_k.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input vector. Must be a non-nullable VectorDomain.
#' @param input_metric Metric on the input domain. Must be LInfDistance
#' @param output_measure One of `MaxDivergence` or `ZeroConcentratedDivergence`
#' @param k Number of indices to select.
#' @param scale Scale for the noise distribution.
#' @param negate Set to true to return bottom k
#' @return Measurement
#' @export
make_noisy_top_k <- function(
  input_domain,
  input_metric,
  output_measure,
  k,
  scale,
  negate = FALSE
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_noisy_top_k", "measurements", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "k", "scale", "negate"),
    list(input_domain, input_metric, output_measure, unbox2(k), unbox2(scale), unbox2(negate))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(k))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(negate))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_noisy_top_k",
    input_domain, input_metric, output_measure, k, scale, negate,
    log_, PACKAGE = "opendp")
  output
}

#' partial noisy top k constructor
#'
#' See documentation for [make_noisy_top_k()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param output_measure One of `MaxDivergence` or `ZeroConcentratedDivergence`
#' @param k Number of indices to select.
#' @param scale Scale for the noise distribution.
#' @param negate Set to true to return bottom k
#' @return Measurement
#' @export
then_noisy_top_k <- function(
  lhs,
  output_measure,
  k,
  scale,
  negate = FALSE
) {

  log_ <- new_constructor_log("then_noisy_top_k", "measurements", new_hashtab(
    list("output_measure", "k", "scale", "negate"),
    list(output_measure, unbox2(k), unbox2(scale), unbox2(negate))
  ))

  make_chain_dyn(
    make_noisy_top_k(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      k = k,
      scale = scale,
      negate = negate),
    lhs,
    log_)
}


#' private quantile constructor
#'
#' Makes a Measurement the computes the quantile of a dataset.
#'
#'
#' Required features: `contrib`
#'
#' [make_private_quantile in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_private_quantile.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Type:    `MI`
#' * Input Metric:   `MO`
#' * Output Measure: `T`
#'
#' @concept measurements
#' @param input_domain Uses a tighter sensitivity when the size of vectors in the input domain is known.
#' @param input_metric Either SymmetricDistance or InsertDeleteDistance.
#' @param output_measure Either MaxDivergence or ZeroConcentratedDivergence.
#' @param candidates Potential quantiles to score
#' @param alpha a value in \eqn{[0, 1]}. Choose 0.5 for median
#' @param scale the scale of the noise added
#' @return Measurement
#' @export
make_private_quantile <- function(
  input_domain,
  input_metric,
  output_measure,
  candidates,
  alpha,
  scale
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- get_atom(get_type(input_domain))
  .T.candidates <- new_runtime_type(origin = "Vec", args = list(.T))

  log_ <- new_constructor_log("make_private_quantile", "measurements", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "candidates", "alpha", "scale"),
    list(input_domain, input_metric, output_measure, candidates, unbox2(alpha), unbox2(scale))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.candidates, inferred = rt_infer(candidates))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(alpha))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_private_quantile",
    input_domain, input_metric, output_measure, candidates, alpha, scale, .T, rt_parse(.T.candidates),
    log_, PACKAGE = "opendp")
  output
}

#' partial private quantile constructor
#'
#' See documentation for [make_private_quantile()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param output_measure Either MaxDivergence or ZeroConcentratedDivergence.
#' @param candidates Potential quantiles to score
#' @param alpha a value in \eqn{[0, 1]}. Choose 0.5 for median
#' @param scale the scale of the noise added
#' @return Measurement
#' @export
then_private_quantile <- function(
  lhs,
  output_measure,
  candidates,
  alpha,
  scale
) {

  log_ <- new_constructor_log("then_private_quantile", "measurements", new_hashtab(
    list("output_measure", "candidates", "alpha", "scale"),
    list(output_measure, candidates, unbox2(alpha), unbox2(scale))
  ))

  make_chain_dyn(
    make_private_quantile(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      candidates = candidates,
      alpha = alpha,
      scale = scale),
    lhs,
    log_)
}


#' randomized response constructor
#'
#' Make a Measurement that implements randomized response on a categorical value.
#'
#'
#' Required features: `contrib`
#'
#' [make_randomized_response in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_randomized_response.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<T>`
#' * Output Type:    `DiscreteDistance`
#' * Input Metric:   `MaxDivergence`
#' * Output Measure: `T`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response/make_randomized_response.pdf)
#'
#' @concept measurements
#' @param categories Set of valid outcomes
#' @param prob Probability of returning the correct answer. Must be in `[1/num_categories, 1]`
#' @param .T Data type of a category.
#' @return Measurement
#' @export
make_randomized_response <- function(
  categories,
  prob,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(categories))
  .T.categories <- new_runtime_type(origin = "Vec", args = list(.T))

  log_ <- new_constructor_log("make_randomized_response", "measurements", new_hashtab(
    list("categories", "prob", "T"),
    list(categories, unbox2(prob), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.categories, inferred = rt_infer(categories))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(prob))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_randomized_response",
    categories, prob, .T, rt_parse(.T.categories),
    log_, PACKAGE = "opendp")
  output
}

#' partial randomized response constructor
#'
#' See documentation for [make_randomized_response()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param categories Set of valid outcomes
#' @param prob Probability of returning the correct answer. Must be in `[1/num_categories, 1]`
#' @param .T Data type of a category.
#' @return Measurement
#' @export
then_randomized_response <- function(
  lhs,
  categories,
  prob,
  .T = NULL
) {

  log_ <- new_constructor_log("then_randomized_response", "measurements", new_hashtab(
    list("categories", "prob", "T"),
    list(categories, unbox2(prob), .T)
  ))

  make_chain_dyn(
    make_randomized_response(
      categories = categories,
      prob = prob,
      .T = .T),
    lhs,
    log_)
}


#' randomized response bitvec constructor
#'
#' Make a Measurement that implements randomized response on a bit vector.
#'
#' This primitive can be useful for implementing RAPPOR.
#'
#'
#' Required features: `contrib`
#'
#' [make_randomized_response_bitvec in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_randomized_response_bitvec.html)
#'
#' **Citations:**
#'
#' * [RAPPOR: Randomized Aggregatable Privacy-Preserving Ordinal Response](https://arxiv.org/abs/1407.6981)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `BitVectorDomain`
#' * Output Type:    `DiscreteDistance`
#' * Input Metric:   `MaxDivergence`
#' * Output Measure: `BitVector`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response_bitvec/make_randomized_response_bitvec.pdf)
#'
#' @concept measurements
#' @param input_domain BitVectorDomain with max_weight
#' @param input_metric DiscreteDistance
#' @param f Per-bit flipping probability. Must be in \eqn{(0, 1]}.
#' @param constant_time Whether to run the Bernoulli samplers in constant time, this is likely to be extremely slow.
#' @return Measurement
#' @export
make_randomized_response_bitvec <- function(
  input_domain,
  input_metric,
  f,
  constant_time = FALSE
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_randomized_response_bitvec", "measurements", new_hashtab(
    list("input_domain", "input_metric", "f", "constant_time"),
    list(input_domain, input_metric, unbox2(f), unbox2(constant_time))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(f))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(constant_time))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_randomized_response_bitvec",
    input_domain, input_metric, f, constant_time,
    log_, PACKAGE = "opendp")
  output
}

#' partial randomized response bitvec constructor
#'
#' See documentation for [make_randomized_response_bitvec()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param f Per-bit flipping probability. Must be in \eqn{(0, 1]}.
#' @param constant_time Whether to run the Bernoulli samplers in constant time, this is likely to be extremely slow.
#' @return Measurement
#' @export
then_randomized_response_bitvec <- function(
  lhs,
  f,
  constant_time = FALSE
) {

  log_ <- new_constructor_log("then_randomized_response_bitvec", "measurements", new_hashtab(
    list("f", "constant_time"),
    list(unbox2(f), unbox2(constant_time))
  ))

  make_chain_dyn(
    make_randomized_response_bitvec(
      output_domain(lhs),
      output_metric(lhs),
      f = f,
      constant_time = constant_time),
    lhs,
    log_)
}


#' randomized response bool constructor
#'
#' Make a Measurement that implements randomized response on a boolean value.
#'
#'
#' Required features: `contrib`
#'
#' [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_randomized_response_bool.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<bool>`
#' * Output Type:    `DiscreteDistance`
#' * Input Metric:   `MaxDivergence`
#' * Output Measure: `bool`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/beta/proofs/rust/src/measurements/randomized_response/make_randomized_response_bool.pdf)
#'
#' @concept measurements
#' @param prob Probability of returning the correct answer. Must be in `[0.5, 1]`
#' @param constant_time Set to true to enable constant time. Slower.
#' @return Measurement
#' @export
make_randomized_response_bool <- function(
  prob,
  constant_time = FALSE
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_randomized_response_bool", "measurements", new_hashtab(
    list("prob", "constant_time"),
    list(unbox2(prob), unbox2(constant_time))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(prob))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(constant_time))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_randomized_response_bool",
    prob, constant_time,
    log_, PACKAGE = "opendp")
  output
}

#' partial randomized response bool constructor
#'
#' See documentation for [make_randomized_response_bool()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param prob Probability of returning the correct answer. Must be in `[0.5, 1]`
#' @param constant_time Set to true to enable constant time. Slower.
#' @return Measurement
#' @export
then_randomized_response_bool <- function(
  lhs,
  prob,
  constant_time = FALSE
) {

  log_ <- new_constructor_log("then_randomized_response_bool", "measurements", new_hashtab(
    list("prob", "constant_time"),
    list(unbox2(prob), unbox2(constant_time))
  ))

  make_chain_dyn(
    make_randomized_response_bool(
      prob = prob,
      constant_time = constant_time),
    lhs,
    log_)
}


#' report noisy max gumbel constructor
#'
#' Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
#'
#'
#' Required features: `contrib`
#'
#' [make_report_noisy_max_gumbel in Rust documentation.](https://docs.rs/opendp/0.14.1-beta.20250916.1/opendp/measurements/fn.make_report_noisy_max_gumbel.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Type:    `LInfDistance<TIA>`
#' * Input Metric:   `MaxDivergence`
#' * Output Measure: `usize`
#'
#' @concept measurements
#' @param input_domain Domain of the input vector. Must be a non-nullable `VectorDomain`
#' @param input_metric Metric on the input domain. Must be `LInfDistance`
#' @param scale Scale for the noise distribution
#' @param optimize Set to "min" to report noisy min
#' @return Measurement
#' @export
make_report_noisy_max_gumbel <- function(
  input_domain,
  input_metric,
  scale,
  optimize = "max"
) {
  .Deprecated(msg = "use `make_noisy_max` instead")
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_report_noisy_max_gumbel", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "optimize"),
    list(input_domain, input_metric, unbox2(scale), unbox2(optimize))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = String, inferred = rt_infer(optimize))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_report_noisy_max_gumbel",
    input_domain, input_metric, scale, optimize,
    log_, PACKAGE = "opendp")
  output
}

#' partial report noisy max gumbel constructor
#'
#' See documentation for [make_report_noisy_max_gumbel()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Scale for the noise distribution
#' @param optimize Set to "min" to report noisy min
#' @return Measurement
#' @export
then_report_noisy_max_gumbel <- function(
  lhs,
  scale,
  optimize = "max"
) {

  log_ <- new_constructor_log("then_report_noisy_max_gumbel", "measurements", new_hashtab(
    list("scale", "optimize"),
    list(unbox2(scale), unbox2(optimize))
  ))

  make_chain_dyn(
    make_report_noisy_max_gumbel(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      optimize = optimize),
    lhs,
    log_)
}
