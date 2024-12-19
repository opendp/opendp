# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' approximate constructor
#'
#' Constructs a new output measurement where the output measure
#' is δ-approximate, where δ=0.
#'
#'
#' Required features: `contrib`
#'
#' [make_approximate in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_approximate.html)
#'
#' @concept combinators
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
make_approximate <- function(
  measurement
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_approximate", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasurement, inferred = rt_infer(measurement))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_approximate",
    measurement,
    log, PACKAGE = "opendp")
  output
}

#' partial approximate constructor
#'
#' See documentation for [make_approximate()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
then_approximate <- function(
  lhs,
  measurement
) {

  log <- new_constructor_log("then_approximate", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  make_chain_dyn(
    make_approximate(
      measurement = measurement),
    lhs,
    log)
}


#' basic composition constructor
#'
#' Construct the DP composition \[`measurement0`, `measurement1`, ...\].
#' Returns a Measurement that when invoked, computes `[measurement0(x), measurement1(x), ...]`
#'
#' All metrics and domains must be equivalent.
#'
#' **Composition Properties**
#'
#' * sequential: all measurements are applied to the same dataset
#' * basic: the composition is the linear sum of the privacy usage of each query
#' * noninteractive: all mechanisms specified up-front (but each can be interactive)
#' * compositor: all privacy parameters specified up-front (via the map)
#'
#'
#' Required features: `contrib`
#'
#' [make_basic_composition in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_basic_composition.html)
#'
#' @concept combinators
#' @param measurements A vector of Measurements to compose.
#' @return Measurement
#' @export
make_basic_composition <- function(
  measurements
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T.measurements <- new_runtime_type(origin = "Vec", args = list(AnyMeasurementPtr))

  log <- new_constructor_log("make_basic_composition", "combinators", new_hashtab(
    list("measurements"),
    list(measurements)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.measurements, inferred = rt_infer(measurements))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_basic_composition",
    measurements, rt_parse(.T.measurements),
    log, PACKAGE = "opendp")
  output
}

#' partial basic composition constructor
#'
#' See documentation for [make_basic_composition()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurements A vector of Measurements to compose.
#' @return Measurement
#' @export
then_basic_composition <- function(
  lhs,
  measurements
) {

  log <- new_constructor_log("then_basic_composition", "combinators", new_hashtab(
    list("measurements"),
    list(measurements)
  ))

  make_chain_dyn(
    make_basic_composition(
      measurements = measurements),
    lhs,
    log)
}


#' chain mt constructor
#'
#' Construct the functional composition (`measurement1` ○ `transformation0`).
#' Returns a Measurement that when invoked, computes `measurement1(transformation0(x))`.
#'
#'
#' Required features: `contrib`
#'
#' [make_chain_mt in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_chain_mt.html)
#'
#' @concept combinators
#' @param measurement1 outer mechanism
#' @param transformation0 inner transformation
#' @return Measurement
#' @export
make_chain_mt <- function(
  measurement1,
  transformation0
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_chain_mt", "combinators", new_hashtab(
    list("measurement1", "transformation0"),
    list(measurement1, transformation0)
  ))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_chain_mt",
    measurement1, transformation0,
    log, PACKAGE = "opendp")
  output
}

#' partial chain mt constructor
#'
#' See documentation for [make_chain_mt()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement1 outer mechanism
#' @param transformation0 inner transformation
#' @return Measurement
#' @export
then_chain_mt <- function(
  lhs,
  measurement1,
  transformation0
) {

  log <- new_constructor_log("then_chain_mt", "combinators", new_hashtab(
    list("measurement1", "transformation0"),
    list(measurement1, transformation0)
  ))

  make_chain_dyn(
    make_chain_mt(
      measurement1 = measurement1,
      transformation0 = transformation0),
    lhs,
    log)
}


#' chain pm constructor
#'
#' Construct the functional composition (`postprocess1` ○ `measurement0`).
#' Returns a Measurement that when invoked, computes `postprocess1(measurement0(x))`.
#' Used to represent non-interactive postprocessing.
#'
#'
#' Required features: `contrib`
#'
#' [make_chain_pm in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_chain_pm.html)
#'
#' @concept combinators
#' @param postprocess1 outer postprocessor
#' @param measurement0 inner measurement/mechanism
#' @return Measurement
#' @export
make_chain_pm <- function(
  postprocess1,
  measurement0
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_chain_pm", "combinators", new_hashtab(
    list("postprocess1", "measurement0"),
    list(postprocess1, measurement0)
  ))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_chain_pm",
    postprocess1, measurement0,
    log, PACKAGE = "opendp")
  output
}

#' partial chain pm constructor
#'
#' See documentation for [make_chain_pm()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param postprocess1 outer postprocessor
#' @param measurement0 inner measurement/mechanism
#' @return Measurement
#' @export
then_chain_pm <- function(
  lhs,
  postprocess1,
  measurement0
) {

  log <- new_constructor_log("then_chain_pm", "combinators", new_hashtab(
    list("postprocess1", "measurement0"),
    list(postprocess1, measurement0)
  ))

  make_chain_dyn(
    make_chain_pm(
      postprocess1 = postprocess1,
      measurement0 = measurement0),
    lhs,
    log)
}


#' chain tt constructor
#'
#' Construct the functional composition (`transformation1` ○ `transformation0`).
#' Returns a Transformation that when invoked, computes `transformation1(transformation0(x))`.
#'
#'
#' Required features: `contrib`
#'
#' [make_chain_tt in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_chain_tt.html)
#'
#' @concept combinators
#' @param transformation1 outer transformation
#' @param transformation0 inner transformation
#' @return Transformation
#' @export
make_chain_tt <- function(
  transformation1,
  transformation0
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_chain_tt", "combinators", new_hashtab(
    list("transformation1", "transformation0"),
    list(transformation1, transformation0)
  ))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_chain_tt",
    transformation1, transformation0,
    log, PACKAGE = "opendp")
  output
}

#' partial chain tt constructor
#'
#' See documentation for [make_chain_tt()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param transformation1 outer transformation
#' @param transformation0 inner transformation
#' @return Transformation
#' @export
then_chain_tt <- function(
  lhs,
  transformation1,
  transformation0
) {

  log <- new_constructor_log("then_chain_tt", "combinators", new_hashtab(
    list("transformation1", "transformation0"),
    list(transformation1, transformation0)
  ))

  make_chain_dyn(
    make_chain_tt(
      transformation1 = transformation1,
      transformation0 = transformation0),
    lhs,
    log)
}


#' fix delta constructor
#'
#' Fix the delta parameter in the privacy map of a `measurement` with a SmoothedMaxDivergence output measure.
#'
#'
#' Required features: `contrib`
#'
#' [make_fix_delta in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_fix_delta.html)
#'
#' @concept combinators
#' @param measurement a measurement with a privacy curve to be fixed
#' @param delta parameter to fix the privacy curve with
#' @return Measurement
#' @export
make_fix_delta <- function(
  measurement,
  delta
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_fix_delta", "combinators", new_hashtab(
    list("measurement", "delta"),
    list(measurement, unbox2(delta))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = f64, inferred = rt_infer(delta))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_fix_delta",
    measurement, delta,
    log, PACKAGE = "opendp")
  output
}

#' partial fix delta constructor
#'
#' See documentation for [make_fix_delta()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement a measurement with a privacy curve to be fixed
#' @param delta parameter to fix the privacy curve with
#' @return Measurement
#' @export
then_fix_delta <- function(
  lhs,
  measurement,
  delta
) {

  log <- new_constructor_log("then_fix_delta", "combinators", new_hashtab(
    list("measurement", "delta"),
    list(measurement, unbox2(delta))
  ))

  make_chain_dyn(
    make_fix_delta(
      measurement = measurement,
      delta = delta),
    lhs,
    log)
}


#' population amplification constructor
#'
#' Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
#' This measurement does not perform any sampling.
#' It is useful when you have a dataset on-hand that is a simple random sample from a larger population.
#'
#' The DIA, DO, MI and MO between the input measurement and amplified output measurement all match.
#'
#'
#' Required features: `contrib`, `honest-but-curious`
#'
#' [make_population_amplification in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_population_amplification.html)
#'
#' **Why honest-but-curious?:**
#'
#' The privacy guarantees are only valid if the input dataset is a simple sample from a population with `population_size` records.
#'
#' @concept combinators
#' @param measurement the computation to amplify
#' @param population_size the size of the population from which the input dataset is a simple sample
#' @return Measurement
#' @export
make_population_amplification <- function(
  measurement,
  population_size
) {
  assert_features("contrib", "honest-but-curious")

  # No type arguments to standardize.
  log <- new_constructor_log("make_population_amplification", "combinators", new_hashtab(
    list("measurement", "population_size"),
    list(measurement, unbox2(population_size))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasurement, inferred = rt_infer(measurement))
  rt_assert_is_similar(expected = usize, inferred = rt_infer(population_size))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_population_amplification",
    measurement, population_size,
    log, PACKAGE = "opendp")
  output
}

#' partial population amplification constructor
#'
#' See documentation for [make_population_amplification()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement the computation to amplify
#' @param population_size the size of the population from which the input dataset is a simple sample
#' @return Measurement
#' @export
then_population_amplification <- function(
  lhs,
  measurement,
  population_size
) {

  log <- new_constructor_log("then_population_amplification", "combinators", new_hashtab(
    list("measurement", "population_size"),
    list(measurement, unbox2(population_size))
  ))

  make_chain_dyn(
    make_population_amplification(
      measurement = measurement,
      population_size = population_size),
    lhs,
    log)
}


#' pureDP to zCDP constructor
#'
#' Constructs a new output measurement where the output measure
#' is casted from `MaxDivergence` to `ZeroConcentratedDivergence`.
#'
#'
#' Required features: `contrib`
#'
#' [make_pureDP_to_zCDP in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_pureDP_to_zCDP.html)
#'
#' **Citations:**
#'
#' - [BS16 Concentrated Differential Privacy: Simplifications, Extensions, and Lower Bounds](https://arxiv.org/pdf/1605.02065.pdf#subsection.3.1)
#'
#' @concept combinators
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
make_pureDP_to_zCDP <- function(
  measurement
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_pureDP_to_zCDP", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasurement, inferred = rt_infer(measurement))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_pureDP_to_zCDP",
    measurement,
    log, PACKAGE = "opendp")
  output
}

#' partial pureDP to zCDP constructor
#'
#' See documentation for [make_pureDP_to_zCDP()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
then_pureDP_to_zCDP <- function(
  lhs,
  measurement
) {

  log <- new_constructor_log("then_pureDP_to_zCDP", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  make_chain_dyn(
    make_pureDP_to_zCDP(
      measurement = measurement),
    lhs,
    log)
}


#' select private candidate constructor
#'
#' Select a private candidate whose score is above a threshold.
#'
#' Given `measurement` that satisfies ε-DP, returns new measurement M' that satisfies 2ε-DP.
#' M' releases the first invocation of `measurement` whose score is above `threshold`.
#'
#' Each time a score is below `threshold`
#' the algorithm may terminate with probability `stop_probability` and return nothing.
#'
#' `measurement` should make releases in the form of (score, candidate).
#' If you are writing a custom scorer measurement in Python,
#' specify the output type as `TO=(float, "ExtrinsicObject")`.
#' This ensures that the float value is accessible to the algorithm.
#' The candidate, left as arbitrary Python data, is held behind the ExtrinsicObject.
#'
#' Algorithm 1 in `Private selection from private candidates <https://arxiv.org/pdf/1811.07971.pdf#page=7>`_ (Liu and Talwar, STOC 2019).
#'
#'
#' Required features: `contrib`
#'
#' [make_select_private_candidate in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_select_private_candidate.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `Option<(f64, TO)>`
#' * Input Metric:   `MI`
#' * Output Measure: `MaxDivergence`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/v0.12.0/proofs/rust/src/combinators/select_private_candidate/make_select_private_candidate.pdf)
#'
#' @concept combinators
#' @param measurement A measurement that releases a 2-tuple of (score, candidate)
#' @param stop_probability The probability of stopping early at any iteration.
#' @param threshold The threshold score. Return immediately if the score is above this threshold.
#' @return A measurement that returns a release from `measurement` whose score is greater than `threshold`, or none.
#' @export
make_select_private_candidate <- function(
  measurement,
  stop_probability,
  threshold
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_select_private_candidate", "combinators", new_hashtab(
    list("measurement", "stop_probability", "threshold"),
    list(measurement, unbox2(stop_probability), unbox2(threshold))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasurement, inferred = rt_infer(measurement))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(stop_probability))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(threshold))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_select_private_candidate",
    measurement, stop_probability, threshold,
    log, PACKAGE = "opendp")
  output
}

#' partial select private candidate constructor
#'
#' See documentation for [make_select_private_candidate()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement A measurement that releases a 2-tuple of (score, candidate)
#' @param stop_probability The probability of stopping early at any iteration.
#' @param threshold The threshold score. Return immediately if the score is above this threshold.
#' @return A measurement that returns a release from `measurement` whose score is greater than `threshold`, or none.
#' @export
then_select_private_candidate <- function(
  lhs,
  measurement,
  stop_probability,
  threshold
) {

  log <- new_constructor_log("then_select_private_candidate", "combinators", new_hashtab(
    list("measurement", "stop_probability", "threshold"),
    list(measurement, unbox2(stop_probability), unbox2(threshold))
  ))

  make_chain_dyn(
    make_select_private_candidate(
      measurement = measurement,
      stop_probability = stop_probability,
      threshold = threshold),
    lhs,
    log)
}


#' sequential composition constructor
#'
#' Construct a Measurement that when invoked,
#' returns a queryable that interactively composes measurements.
#'
#' **Composition Properties**
#'
#' * sequential: all measurements are applied to the same dataset
#' * basic: the composition is the linear sum of the privacy usage of each query
#' * interactive: mechanisms can be specified based on answers to previous queries
#' * compositor: all privacy parameters specified up-front
#'
#' If the privacy measure supports concurrency,
#' this compositor allows you to spawn multiple interactive mechanisms
#' and interleave your queries amongst them.
#'
#'
#' Required features: `contrib`
#'
#' [make_sequential_composition in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_sequential_composition.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DI`
#' * Output Type:    `Queryable<Measurement<DI, TO, MI, MO>, TO>`
#' * Input Metric:   `MI`
#' * Output Measure: `MO`
#'
#' @concept combinators
#' @param input_domain indicates the space of valid input datasets
#' @param input_metric how distances are measured between members of the input domain
#' @param output_measure how privacy is measured
#' @param d_in maximum distance between adjacent input datasets
#' @param d_mids maximum privacy expenditure of each query
#' @return Measurement
#' @export
make_sequential_composition <- function(
  input_domain,
  input_metric,
  output_measure,
  d_in,
  d_mids
) {
  assert_features("contrib")

  # Standardize type arguments.
  .QO <- get_distance_type(output_measure)
  .T.d_in <- get_distance_type(input_metric)
  .T.d_mids <- new_runtime_type(origin = "Vec", args = list(.QO))

  log <- new_constructor_log("make_sequential_composition", "combinators", new_hashtab(
    list("input_domain", "input_metric", "output_measure", "d_in", "d_mids"),
    list(input_domain, input_metric, output_measure, d_in, d_mids)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.d_in, inferred = rt_infer(d_in))
  rt_assert_is_similar(expected = .T.d_mids, inferred = rt_infer(d_mids))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_sequential_composition",
    input_domain, input_metric, output_measure, d_in, d_mids, .QO, rt_parse(.T.d_in), rt_parse(.T.d_mids),
    log, PACKAGE = "opendp")
  output
}

#' partial sequential composition constructor
#'
#' See documentation for [make_sequential_composition()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param output_measure how privacy is measured
#' @param d_in maximum distance between adjacent input datasets
#' @param d_mids maximum privacy expenditure of each query
#' @return Measurement
#' @export
then_sequential_composition <- function(
  lhs,
  output_measure,
  d_in,
  d_mids
) {

  log <- new_constructor_log("then_sequential_composition", "combinators", new_hashtab(
    list("output_measure", "d_in", "d_mids"),
    list(output_measure, d_in, d_mids)
  ))

  make_chain_dyn(
    make_sequential_composition(
      output_domain(lhs),
      output_metric(lhs),
      output_measure = output_measure,
      d_in = d_in,
      d_mids = d_mids),
    lhs,
    log)
}


#' zCDP to approxDP constructor
#'
#' Constructs a new output measurement where the output measure
#' is casted from `ZeroConcentratedDivergence` to `SmoothedMaxDivergence`.
#'
#'
#' Required features: `contrib`
#'
#' [make_zCDP_to_approxDP in Rust documentation.](https://docs.rs/opendp/0.12.0/opendp/combinators/fn.make_zCDP_to_approxDP.html)
#'
#' @concept combinators
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
make_zCDP_to_approxDP <- function(
  measurement
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log <- new_constructor_log("make_zCDP_to_approxDP", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasurement, inferred = rt_infer(measurement))

  # Call wrapper function.
  output <- .Call(
    "combinators__make_zCDP_to_approxDP",
    measurement,
    log, PACKAGE = "opendp")
  output
}

#' partial zCDP to approxDP constructor
#'
#' See documentation for [make_zCDP_to_approxDP()] for details.
#'
#' @concept combinators
#' @param lhs The prior transformation or metric space.
#' @param measurement a measurement with a privacy measure to be casted
#' @return Measurement
#' @export
then_zCDP_to_approxDP <- function(
  lhs,
  measurement
) {

  log <- new_constructor_log("then_zCDP_to_approxDP", "combinators", new_hashtab(
    list("measurement"),
    list(measurement)
  ))

  make_chain_dyn(
    make_zCDP_to_approxDP(
      measurement = measurement),
    lhs,
    log)
}
