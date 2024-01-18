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
#' [make_alp_queryable in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_alp_queryable.html)
#'
#' **Citations:**
#'
#' * [ALP21 Differentially Private Sparse Vectors with Low Error, Optimal Space, and Fast Access](https://arxiv.org/abs/2106.10068) Algorithm 4
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `MapDomain<AtomDomain<K>, AtomDomain<CI>>`
#' * Output Type:    `Queryable<K, CO>`
#' * Input Metric:   `L1Distance<CI>`
#' * Output Measure: `MaxDivergence<CO>`
#'
#' @concept measurements
#' @param input_domain undocumented
#' @param input_metric undocumented
#' @param scale Privacy loss parameter. This is equal to epsilon/sensitivity.
#' @param total_limit Either the true value or an upper bound estimate of the sum of all values in the input.
#' @param value_limit Upper bound on individual values (referred to as β). Entries above β are clamped.
#' @param size_factor Optional multiplier (default of 50) for setting the size of the projection.
#' @param alpha Optional parameter (default of 4) for scaling and determining p in randomized response step.
#' @param .CO undocumented
#' @return Measurement
#' @export
make_alp_queryable <- function(
  input_domain,
  input_metric,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L,
  .CO = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .CO <- parse_or_infer(type_name = .CO, public_example = scale)
  .CI <- get_value_type(get_carrier_type(input_domain))
  .T.value_limit <- new_runtime_type(origin = "Option", args = list(.CI))
  .T.size_factor <- new_runtime_type(origin = "Option", args = list(u32))
  .T.alpha <- new_runtime_type(origin = "Option", args = list(u32))

  log <- new_constructor_log("make_alp_queryable", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "total_limit", "value_limit", "size_factor", "alpha", "CO"),
    list(input_domain, input_metric, scale, total_limit, value_limit, size_factor, alpha, .CO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .CO, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .CI, inferred = rt_infer(total_limit))
  rt_assert_is_similar(expected = .T.value_limit, inferred = rt_infer(value_limit))
  rt_assert_is_similar(expected = .T.size_factor, inferred = rt_infer(size_factor))
  rt_assert_is_similar(expected = .T.alpha, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_alp_queryable",
    input_domain, input_metric, scale, total_limit, value_limit, size_factor, alpha, .CO, .CI, rt_parse(.T.value_limit), rt_parse(.T.size_factor), rt_parse(.T.alpha),
    log, PACKAGE = "opendp")
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
#' @param .CO undocumented
#' @return Measurement
#' @export
then_alp_queryable <- function(
  lhs,
  scale,
  total_limit,
  value_limit = NULL,
  size_factor = 50L,
  alpha = 4L,
  .CO = NULL
) {

  log <- new_constructor_log("then_alp_queryable", "measurements", new_hashtab(
    list("scale", "total_limit", "value_limit", "size_factor", "alpha", "CO"),
    list(scale, total_limit, value_limit, size_factor, alpha, .CO)
  ))

  make_chain_dyn(
    make_alp_queryable(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      total_limit = total_limit,
      value_limit = value_limit,
      size_factor = size_factor,
      alpha = alpha,
      .CO = .CO),
    lhs,
    log)
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
#' [make_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_gaussian.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MO`
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @examples
#' library(opendp)
#' enable_features("contrib")
#' gaussian <- make_gaussian(
#'   atom_domain(.T = f64),
#'   absolute_distance(.T = f64),
#'   scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Or, more readably, define the space and then chain:
#' space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Sensitivity of this measurment:
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
  .MO = "ZeroConcentratedDivergence<.QO>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO, generics = list(".QO"))
  .QO <- get_atom_or_infer(.MO, scale)
  .MO <- rt_substitute(.MO, .QO = .QO)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log <- new_constructor_log("make_gaussian", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "k", "MO"),
    list(input_domain, input_metric, scale, k, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_gaussian",
    input_domain, input_metric, scale, k, .MO, .QO, rt_parse(.T.k),
    log, PACKAGE = "opendp")
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
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @examples
#' library(opendp)
#' enable_features("contrib")
#' gaussian <- make_gaussian(
#'   atom_domain(.T = f64),
#'   absolute_distance(.T = f64),
#'   scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Or, more readably, define the space and then chain:
#' space <- c(atom_domain(.T = f64), absolute_distance(.T = f64))
#' gaussian <- space |> then_gaussian(scale = 1.0)
#' gaussian(arg = 100.0)
#'
#' # Sensitivity of this measurment:
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
  .MO = "ZeroConcentratedDivergence<.QO>"
) {

  log <- new_constructor_log("then_gaussian", "measurements", new_hashtab(
    list("scale", "k", "MO"),
    list(scale, k, .MO)
  ))

  make_chain_dyn(
    make_gaussian(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      k = k,
      .MO = .MO),
    lhs,
    log)
}


#' geometric constructor
#'
#' Equivalent to `make_laplace` but restricted to an integer support.
#' Can specify `bounds` to run the algorithm in near constant-time.
#'
#' [make_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_geometric.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' @concept measurements
#' @param input_domain undocumented
#' @param input_metric undocumented
#' @param scale undocumented
#' @param bounds undocumented
#' @param .QO undocumented
#' @return Measurement
#' @export
make_geometric <- function(
  input_domain,
  input_metric,
  scale,
  bounds = NULL,
  .QO = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .QO <- parse_or_infer(type_name = .QO, public_example = scale)
  .T <- get_atom(get_carrier_type(input_domain))
  .OptionT <- new_runtime_type(origin = "Option", args = list(new_runtime_type(origin = "Tuple", args = list(.T, .T))))

  log <- new_constructor_log("make_geometric", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "bounds", "QO"),
    list(input_domain, input_metric, scale, bounds, .QO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .OptionT, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_geometric",
    input_domain, input_metric, scale, bounds, .QO, .T, .OptionT,
    log, PACKAGE = "opendp")
  output
}

#' partial geometric constructor
#'
#' See documentation for [make_geometric()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale undocumented
#' @param bounds undocumented
#' @param .QO undocumented
#' @return Measurement
#' @export
then_geometric <- function(
  lhs,
  scale,
  bounds = NULL,
  .QO = NULL
) {

  log <- new_constructor_log("then_geometric", "measurements", new_hashtab(
    list("scale", "bounds", "QO"),
    list(scale, bounds, .QO)
  ))

  make_chain_dyn(
    make_geometric(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      bounds = bounds,
      .QO = .QO),
    lhs,
    log)
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
#' [make_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_laplace.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#' * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' @concept measurements
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param k The noise granularity in terms of 2^k, only valid for domains over floats.
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
make_laplace <- function(
  input_domain,
  input_metric,
  scale,
  k = NULL,
  .QO = "float"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .QO <- rt_parse(type_name = .QO)
  .T.scale <- get_atom(.QO)
  .T.k <- new_runtime_type(origin = "Option", args = list(i32))

  log <- new_constructor_log("make_laplace", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "k", "QO"),
    list(input_domain, input_metric, scale, k, .QO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.scale, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .T.k, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_laplace",
    input_domain, input_metric, scale, k, .QO, rt_parse(.T.scale), rt_parse(.T.k),
    log, PACKAGE = "opendp")
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
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
then_laplace <- function(
  lhs,
  scale,
  k = NULL,
  .QO = "float"
) {

  log <- new_constructor_log("then_laplace", "measurements", new_hashtab(
    list("scale", "k", "QO"),
    list(scale, k, .QO)
  ))

  make_chain_dyn(
    make_laplace(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      k = k,
      .QO = .QO),
    lhs,
    log)
}


#' laplace threshold constructor
#'
#' Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#' [make_laplace_threshold in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_laplace_threshold.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
#' * Output Type:    `HashMap<TK, TV>`
#' * Input Metric:   `L1Distance<TV>`
#' * Output Measure: `FixedSmoothedMaxDivergence<TV>`
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric for the input domain.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
make_laplace_threshold <- function(
  input_domain,
  input_metric,
  scale,
  threshold,
  k = -1074L
) {
  assert_features("contrib", "floating-point")

  # Standardize type arguments.
  .TV <- get_distance_type(input_metric)

  log <- new_constructor_log("make_laplace_threshold", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "threshold", "k"),
    list(input_domain, input_metric, scale, threshold, unbox2(k))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TV, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = .TV, inferred = rt_infer(threshold))
  rt_assert_is_similar(expected = i32, inferred = rt_infer(k))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_laplace_threshold",
    input_domain, input_metric, scale, threshold, k, .TV,
    log, PACKAGE = "opendp")
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
#' @return Measurement
#' @export
then_laplace_threshold <- function(
  lhs,
  scale,
  threshold,
  k = -1074L
) {

  log <- new_constructor_log("then_laplace_threshold", "measurements", new_hashtab(
    list("scale", "threshold", "k"),
    list(scale, threshold, unbox2(k))
  ))

  make_chain_dyn(
    make_laplace_threshold(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      threshold = threshold,
      k = k),
    lhs,
    log)
}


#' randomized response constructor
#'
#' Make a Measurement that implements randomized response on a categorical value.
#'
#' [make_randomized_response in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<T>`
#' * Output Type:    `T`
#' * Input Metric:   `DiscreteDistance`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' @concept measurements
#' @param categories Set of valid outcomes
#' @param prob Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
#' @param constant_time Set to true to enable constant time. Slower.
#' @param .T Data type of a category.
#' @param .QO Data type of probability and output distance.
#' @return Measurement
#' @export
make_randomized_response <- function(
  categories,
  prob,
  constant_time = FALSE,
  .T = NULL,
  .QO = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(categories))
  .QO <- parse_or_infer(type_name = .QO, public_example = prob)
  .T.categories <- new_runtime_type(origin = "Vec", args = list(.T))

  log <- new_constructor_log("make_randomized_response", "measurements", new_hashtab(
    list("categories", "prob", "constant_time", "T", "QO"),
    list(categories, prob, unbox2(constant_time), .T, .QO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.categories, inferred = rt_infer(categories))
  rt_assert_is_similar(expected = .QO, inferred = rt_infer(prob))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(constant_time))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_randomized_response",
    categories, prob, constant_time, .T, .QO, rt_parse(.T.categories),
    log, PACKAGE = "opendp")
  output
}

#' partial randomized response constructor
#'
#' See documentation for [make_randomized_response()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param categories Set of valid outcomes
#' @param prob Probability of returning the correct answer. Must be in `[1/num_categories, 1)`
#' @param constant_time Set to true to enable constant time. Slower.
#' @param .T Data type of a category.
#' @param .QO Data type of probability and output distance.
#' @return Measurement
#' @export
then_randomized_response <- function(
  lhs,
  categories,
  prob,
  constant_time = FALSE,
  .T = NULL,
  .QO = NULL
) {

  log <- new_constructor_log("then_randomized_response", "measurements", new_hashtab(
    list("categories", "prob", "constant_time", "T", "QO"),
    list(categories, prob, unbox2(constant_time), .T, .QO)
  ))

  make_chain_dyn(
    make_randomized_response(
      categories = categories,
      prob = prob,
      constant_time = constant_time,
      .T = .T,
      .QO = .QO),
    lhs,
    log)
}


#' randomized response bool constructor
#'
#' Make a Measurement that implements randomized response on a boolean value.
#'
#' [make_randomized_response_bool in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_randomized_response_bool.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<bool>`
#' * Output Type:    `bool`
#' * Input Metric:   `DiscreteDistance`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/randomized_response/make_randomized_response_bool.pdf)
#'
#' @concept measurements
#' @param prob Probability of returning the correct answer. Must be in `[0.5, 1)`
#' @param constant_time Set to true to enable constant time. Slower.
#' @param .QO Data type of probability and output distance.
#' @return Measurement
#' @export
make_randomized_response_bool <- function(
  prob,
  constant_time = FALSE,
  .QO = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .QO <- parse_or_infer(type_name = .QO, public_example = prob)

  log <- new_constructor_log("make_randomized_response_bool", "measurements", new_hashtab(
    list("prob", "constant_time", "QO"),
    list(prob, unbox2(constant_time), .QO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .QO, inferred = rt_infer(prob))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(constant_time))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_randomized_response_bool",
    prob, constant_time, .QO,
    log, PACKAGE = "opendp")
  output
}

#' partial randomized response bool constructor
#'
#' See documentation for [make_randomized_response_bool()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param prob Probability of returning the correct answer. Must be in `[0.5, 1)`
#' @param constant_time Set to true to enable constant time. Slower.
#' @param .QO Data type of probability and output distance.
#' @return Measurement
#' @export
then_randomized_response_bool <- function(
  lhs,
  prob,
  constant_time = FALSE,
  .QO = NULL
) {

  log <- new_constructor_log("then_randomized_response_bool", "measurements", new_hashtab(
    list("prob", "constant_time", "QO"),
    list(prob, unbox2(constant_time), .QO)
  ))

  make_chain_dyn(
    make_randomized_response_bool(
      prob = prob,
      constant_time = constant_time,
      .QO = .QO),
    lhs,
    log)
}


#' report noisy max gumbel constructor
#'
#' Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
#'
#' [make_report_noisy_max_gumbel in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_report_noisy_max_gumbel.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Type:    `usize`
#' * Input Metric:   `LInfDistance<TIA>`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/gumbel_max/make_report_noisy_max_gumbel.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input vector. Must be a non-nullable VectorDomain.
#' @param input_metric Metric on the input domain. Must be LInfDistance
#' @param scale Higher scales are more private.
#' @param optimize Indicate whether to privately return the "Max" or "Min"
#' @param .QO Output Distance Type.
#' @return Measurement
#' @export
make_report_noisy_max_gumbel <- function(
  input_domain,
  input_metric,
  scale,
  optimize,
  .QO = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .QO <- parse_or_infer(type_name = .QO, public_example = scale)

  log <- new_constructor_log("make_report_noisy_max_gumbel", "measurements", new_hashtab(
    list("input_domain", "input_metric", "scale", "optimize", "QO"),
    list(input_domain, input_metric, scale, unbox2(optimize), .QO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))
  rt_assert_is_similar(expected = String, inferred = rt_infer(optimize))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_report_noisy_max_gumbel",
    input_domain, input_metric, scale, optimize, .QO,
    log, PACKAGE = "opendp")
  output
}

#' partial report noisy max gumbel constructor
#'
#' See documentation for [make_report_noisy_max_gumbel()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param scale Higher scales are more private.
#' @param optimize Indicate whether to privately return the "Max" or "Min"
#' @param .QO Output Distance Type.
#' @return Measurement
#' @export
then_report_noisy_max_gumbel <- function(
  lhs,
  scale,
  optimize,
  .QO = NULL
) {

  log <- new_constructor_log("then_report_noisy_max_gumbel", "measurements", new_hashtab(
    list("scale", "optimize", "QO"),
    list(scale, unbox2(optimize), .QO)
  ))

  make_chain_dyn(
    make_report_noisy_max_gumbel(
      output_domain(lhs),
      output_metric(lhs),
      scale = scale,
      optimize = optimize,
      .QO = .QO),
    lhs,
    log)
}


#' tulap constructor
#'
#' Make a Measurement that adds noise from the Tulap distribution to the input.
#'
#' [make_tulap in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_tulap.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<f64>>`
#' * Output Type:    `Vec<f64>`
#' * Input Metric:   `PartitionDistance<AbsoluteDistance<f64>>`
#' * Output Measure: `FixedSmoothedMaxDivergence<f64>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/tulap/make_tulap.pdf)
#'
#' @concept measurements
#' @param input_domain Domain of the input.
#' @param input_metric Metric of the input.
#' @param epsilon Privacy parameter ε.
#' @param delta Privacy parameter δ.
#' @return Measurement
#' @export
make_tulap <- function(
  input_domain,
  input_metric,
  epsilon,
  delta
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_tulap", "measurements", new_hashtab(
    list("input_domain", "input_metric", "epsilon", "delta"),
    list(input_domain, input_metric, unbox2(epsilon), unbox2(delta))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(epsilon))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(delta))

  # Call wrapper function.
  output <- .Call(
    "measurements__make_tulap",
    input_domain, input_metric, epsilon, delta,
    log, PACKAGE = "opendp")
  output
}

#' partial tulap constructor
#'
#' See documentation for [make_tulap()] for details.
#'
#' @concept measurements
#' @param lhs The prior transformation or metric space.
#' @param epsilon Privacy parameter ε.
#' @param delta Privacy parameter δ.
#' @return Measurement
#' @export
then_tulap <- function(
  lhs,
  epsilon,
  delta
) {

  log <- new_constructor_log("then_tulap", "measurements", new_hashtab(
    list("epsilon", "delta"),
    list(unbox2(epsilon), unbox2(delta))
  ))

  make_chain_dyn(
    make_tulap(
      output_domain(lhs),
      output_metric(lhs),
      epsilon = epsilon,
      delta = delta),
    lhs,
    log)
}
