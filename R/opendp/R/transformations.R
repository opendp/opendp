# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Returns an approximation to the ideal `branching_factor` for a dataset of a given size,
#' that minimizes error in cdf and quantile estimates based on b-ary trees.
#'
#'
#' Required features: `contrib`
#'
#' [choose_branching_factor in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.choose_branching_factor.html)
#'
#' **Citations:**
#'
#' * [QYL13 Understanding Hierarchical Methods for Differentially Private Histograms](http://www.vldb.org/pvldb/vol6/p1954-qardaji.pdf)
#'
#' @concept transformations
#' @param size_guess A guess at the size of your dataset.
#' @return int
#' @export
choose_branching_factor <- function(
  size_guess
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("choose_branching_factor", "transformations", new_hashtab(
    list("size_guess"),
    list(unbox2(size_guess))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = u32, inferred = rt_infer(size_guess))

  # Call wrapper function.
  output <- .Call(
    "transformations__choose_branching_factor",
    size_guess,
    log_, PACKAGE = "opendp")
  output
}


#' b ary tree constructor
#'
#' Expand a vector of counts into a b-ary tree of counts,
#' where each branch is the sum of its `b` immediate children.
#'
#'
#' Required features: `contrib`
#'
#' [make_b_ary_tree in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_b_ary_tree.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TA>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param leaf_count The number of leaf nodes in the b-ary tree.
#' @param branching_factor The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
#' @return Transformation
#' @export
make_b_ary_tree <- function(
  input_domain,
  input_metric,
  leaf_count,
  branching_factor
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_b_ary_tree", "transformations", new_hashtab(
    list("input_domain", "input_metric", "leaf_count", "branching_factor"),
    list(input_domain, input_metric, unbox2(leaf_count), unbox2(branching_factor))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = u32, inferred = rt_infer(leaf_count))
  rt_assert_is_similar(expected = u32, inferred = rt_infer(branching_factor))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_b_ary_tree",
    input_domain, input_metric, leaf_count, branching_factor,
    log_, PACKAGE = "opendp")
  output
}

#' partial b ary tree constructor
#'
#' See documentation for [make_b_ary_tree()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param leaf_count The number of leaf nodes in the b-ary tree.
#' @param branching_factor The number of children on each branch of the resulting tree. Larger branching factors result in shallower trees.
#' @return Transformation
#' @export
then_b_ary_tree <- function(
  lhs,
  leaf_count,
  branching_factor
) {

  log_ <- new_constructor_log("then_b_ary_tree", "transformations", new_hashtab(
    list("leaf_count", "branching_factor"),
    list(unbox2(leaf_count), unbox2(branching_factor))
  ))

  make_chain_dyn(
    make_b_ary_tree(
      output_domain(lhs),
      output_metric(lhs),
      leaf_count = leaf_count,
      branching_factor = branching_factor),
    lhs,
    log_)
}


#' bounded float checked sum constructor
#'
#' Make a Transformation that computes the sum of bounded data with known dataset size.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#' Use `make_clamp` to bound data and `make_resize` to establish dataset size.
#'
#' | S (summation algorithm) | input type     |
#' | ----------------------- | -------------- |
#' | `Sequential<S::Item>`   | `Vec<S::Item>` |
#' | `Pairwise<S::Item>`     | `Vec<S::Item>` |
#'
#' `S::Item` is the type of all of the following:
#' each bound, each element in the input data, the output data, and the output sensitivity.
#'
#' For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
#' set `S` to `Pairwise<f32>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_bounded_float_checked_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param size_limit Upper bound on number of records to keep in the input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
make_bounded_float_checked_sum <- function(
  size_limit,
  bounds,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom_or_infer(.S, get_first(bounds))
  .S <- rt_substitute(.S, .T = .T)
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_bounded_float_checked_sum", "transformations", new_hashtab(
    list("size_limit", "bounds", "S"),
    list(unbox2(size_limit), lapply(bounds, unbox2), .S)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size_limit))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_bounded_float_checked_sum",
    size_limit, bounds, .S, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial bounded float checked sum constructor
#'
#' See documentation for [make_bounded_float_checked_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size_limit Upper bound on number of records to keep in the input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
then_bounded_float_checked_sum <- function(
  lhs,
  size_limit,
  bounds,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_bounded_float_checked_sum", "transformations", new_hashtab(
    list("size_limit", "bounds", "S"),
    list(unbox2(size_limit), lapply(bounds, unbox2), .S)
  ))

  make_chain_dyn(
    make_bounded_float_checked_sum(
      size_limit = size_limit,
      bounds = bounds,
      .S = .S),
    lhs,
    log_)
}


#' bounded float ordered sum constructor
#'
#' Make a Transformation that computes the sum of bounded floats with known ordering.
#'
#' Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
#' You may need to use `make_ordered_random` to impose an ordering on the data.
#' The utility loss from overestimating the `size_limit` is small.
#'
#' | S (summation algorithm) | input type     |
#' | ----------------------- | -------------- |
#' | `Sequential<S::Item>`   | `Vec<S::Item>` |
#' | `Pairwise<S::Item>`     | `Vec<S::Item>` |
#'
#' `S::Item` is the type of all of the following:
#' each bound, each element in the input data, the output data, and the output sensitivity.
#'
#' For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
#' set `S` to `Pairwise<f32>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_bounded_float_ordered_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `InsertDeleteDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param size_limit Upper bound on the number of records in input data. Used to bound sensitivity.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
make_bounded_float_ordered_sum <- function(
  size_limit,
  bounds,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom_or_infer(.S, get_first(bounds))
  .S <- rt_substitute(.S, .T = .T)
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_bounded_float_ordered_sum", "transformations", new_hashtab(
    list("size_limit", "bounds", "S"),
    list(unbox2(size_limit), lapply(bounds, unbox2), .S)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size_limit))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_bounded_float_ordered_sum",
    size_limit, bounds, .S, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial bounded float ordered sum constructor
#'
#' See documentation for [make_bounded_float_ordered_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size_limit Upper bound on the number of records in input data. Used to bound sensitivity.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
then_bounded_float_ordered_sum <- function(
  lhs,
  size_limit,
  bounds,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_bounded_float_ordered_sum", "transformations", new_hashtab(
    list("size_limit", "bounds", "S"),
    list(unbox2(size_limit), lapply(bounds, unbox2), .S)
  ))

  make_chain_dyn(
    make_bounded_float_ordered_sum(
      size_limit = size_limit,
      bounds = bounds,
      .S = .S),
    lhs,
    log_)
}


#' bounded int monotonic sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints,
#' where all values share the same sign.
#'
#'
#' Required features: `contrib`
#'
#' [make_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_bounded_int_monotonic_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_bounded_int_monotonic_sum <- function(
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_bounded_int_monotonic_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_bounded_int_monotonic_sum",
    bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial bounded int monotonic sum constructor
#'
#' See documentation for [make_bounded_int_monotonic_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_bounded_int_monotonic_sum <- function(
  lhs,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_bounded_int_monotonic_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_bounded_int_monotonic_sum(
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' bounded int ordered sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints.
#' You may need to use `make_ordered_random` to impose an ordering on the data.
#'
#'
#' Required features: `contrib`
#'
#' [make_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_bounded_int_ordered_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `InsertDeleteDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_bounded_int_ordered_sum <- function(
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_bounded_int_ordered_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_bounded_int_ordered_sum",
    bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial bounded int ordered sum constructor
#'
#' See documentation for [make_bounded_int_ordered_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_bounded_int_ordered_sum <- function(
  lhs,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_bounded_int_ordered_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_bounded_int_ordered_sum(
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' bounded int split sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints.
#' Adds the saturating sum of the positives to the saturating sum of the negatives.
#'
#'
#' Required features: `contrib`
#'
#' [make_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_bounded_int_split_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_bounded_int_split_sum <- function(
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_bounded_int_split_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_bounded_int_split_sum",
    bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial bounded int split sum constructor
#'
#' See documentation for [make_bounded_int_split_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_bounded_int_split_sum <- function(
  lhs,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_bounded_int_split_sum", "transformations", new_hashtab(
    list("bounds", "T"),
    list(lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_bounded_int_split_sum(
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' cast constructor
#'
#' Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
#' For each element, failure to parse results in `None`, else `Some(out)`.
#'
#' Can be chained with `make_impute_constant` or `make_drop_null` to handle nullity.
#'
#'
#' Required features: `contrib`
#'
#' [make_cast in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_cast.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<OptionDomain<AtomDomain<TOA>>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
make_cast <- function(
  input_domain,
  input_metric,
  .TOA
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TOA <- rt_parse(type_name = .TOA)

  log_ <- new_constructor_log("make_cast", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TOA"),
    list(input_domain, input_metric, .TOA)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_cast",
    input_domain, input_metric, .TOA,
    log_, PACKAGE = "opendp")
  output
}

#' partial cast constructor
#'
#' See documentation for [make_cast()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
then_cast <- function(
  lhs,
  .TOA
) {

  log_ <- new_constructor_log("then_cast", "transformations", new_hashtab(
    list("TOA"),
    list(.TOA)
  ))

  make_chain_dyn(
    make_cast(
      output_domain(lhs),
      output_metric(lhs),
      .TOA = .TOA),
    lhs,
    log_)
}


#' cast default constructor
#'
#' Make a Transformation that casts a vector of data from type `TIA` to type `TOA`.
#' Any element that fails to cast is filled with default.
#'
#'
#' | `TIA`  | `TIA::default()` |
#' | ------ | ---------------- |
#' | float  | `0.`             |
#' | int    | `0`              |
#' | string | `""`             |
#' | bool   | `false`          |
#'
#'
#' Required features: `contrib`
#'
#' [make_cast_default in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_cast_default.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
make_cast_default <- function(
  input_domain,
  input_metric,
  .TOA
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TOA <- rt_parse(type_name = .TOA, generics = list(".TIA", ".M"))
  .TIA <- get_atom(get_type(input_domain))
  .M <- get_type(input_metric)
  .TOA <- rt_substitute(.TOA, .TIA = .TIA, .M = .M)

  log_ <- new_constructor_log("make_cast_default", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TOA"),
    list(input_domain, input_metric, .TOA)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_cast_default",
    input_domain, input_metric, .TOA, .TIA, .M,
    log_, PACKAGE = "opendp")
  output
}

#' partial cast default constructor
#'
#' See documentation for [make_cast_default()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
then_cast_default <- function(
  lhs,
  .TOA
) {

  log_ <- new_constructor_log("then_cast_default", "transformations", new_hashtab(
    list("TOA"),
    list(.TOA)
  ))

  make_chain_dyn(
    make_cast_default(
      output_domain(lhs),
      output_metric(lhs),
      .TOA = .TOA),
    lhs,
    log_)
}


#' cast inherent constructor
#'
#' Make a Transformation that casts a vector of data from type `TIA` to a type that can represent nullity `TOA`.
#' If cast fails, fill with `TOA`'s null value.
#'
#' | `TIA`  | `TIA::default()` |
#' | ------ | ---------------- |
#' | float  | NaN              |
#'
#'
#' Required features: `contrib`
#'
#' [make_cast_inherent in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_cast_inherent.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
make_cast_inherent <- function(
  input_domain,
  input_metric,
  .TOA
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TOA <- rt_parse(type_name = .TOA)

  log_ <- new_constructor_log("make_cast_inherent", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TOA"),
    list(input_domain, input_metric, .TOA)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_cast_inherent",
    input_domain, input_metric, .TOA,
    log_, PACKAGE = "opendp")
  output
}

#' partial cast inherent constructor
#'
#' See documentation for [make_cast_inherent()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
then_cast_inherent <- function(
  lhs,
  .TOA
) {

  log_ <- new_constructor_log("then_cast_inherent", "transformations", new_hashtab(
    list("TOA"),
    list(.TOA)
  ))

  make_chain_dyn(
    make_cast_inherent(
      output_domain(lhs),
      output_metric(lhs),
      .TOA = .TOA),
    lhs,
    log_)
}


#' cdf constructor
#'
#' Postprocess a noisy array of float summary counts into a cumulative distribution.
#'
#'
#' Required features: `contrib`
#'
#' [make_cdf in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_cdf.html)
#'
#' **Supporting Elements:**
#'
#' * Input Type:     `Vec<TA>`
#' * Output Type:    `Vec<TA>`
#'
#' @concept transformations
#' @param .TA Atomic Type. One of `f32` or `f64`
#' @return Function
#' @export
make_cdf <- function(
  .TA = "float"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TA <- rt_parse(type_name = .TA)

  log_ <- new_constructor_log("make_cdf", "transformations", new_hashtab(
    list("TA"),
    list(.TA)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_cdf",
    .TA,
    log_, PACKAGE = "opendp")
  output
}

#' partial cdf constructor
#'
#' See documentation for [make_cdf()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TA Atomic Type. One of `f32` or `f64`
#' @return Function
#' @export
then_cdf <- function(
  lhs,
  .TA = "float"
) {

  log_ <- new_constructor_log("then_cdf", "transformations", new_hashtab(
    list("TA"),
    list(.TA)
  ))

  make_chain_dyn(
    make_cdf(
      .TA = .TA),
    lhs,
    log_)
}


#' clamp constructor
#'
#' Make a Transformation that clamps numeric data in `Vec<TA>` to `bounds`.
#'
#' If datum is less than lower, let datum be lower.
#' If datum is greater than upper, let datum be upper.
#'
#'
#' Required features: `contrib`
#'
#' [make_clamp in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_clamp.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TA>>`
#' * Output Metric:  `M`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/clamp/make_clamp.pdf)
#'
#' @concept transformations
#' @param input_domain Domain of input data.
#' @param input_metric Metric on input domain.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
make_clamp <- function(
  input_domain,
  input_metric,
  bounds
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TA <- get_atom(get_type(input_domain))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.TA, .TA))

  log_ <- new_constructor_log("make_clamp", "transformations", new_hashtab(
    list("input_domain", "input_metric", "bounds"),
    list(input_domain, input_metric, lapply(bounds, unbox2))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_clamp",
    input_domain, input_metric, bounds, .TA, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial clamp constructor
#'
#' See documentation for [make_clamp()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
then_clamp <- function(
  lhs,
  bounds
) {

  log_ <- new_constructor_log("then_clamp", "transformations", new_hashtab(
    list("bounds"),
    list(lapply(bounds, unbox2))
  ))

  make_chain_dyn(
    make_clamp(
      output_domain(lhs),
      output_metric(lhs),
      bounds = bounds),
    lhs,
    log_)
}


#' consistent b ary tree constructor
#'
#' Postprocessor that makes a noisy b-ary tree internally consistent, and returns the leaf layer.
#'
#' The input argument of the function is a balanced `b`-ary tree implicitly stored in breadth-first order
#' Tree is assumed to be complete, as in, all leaves on the last layer are on the left.
#' Non-existent leaves are assumed to be zero.
#'
#' The output remains consistent even when leaf nodes are missing.
#' This is due to an adjustment to the original algorithm to apportion corrections to children relative to their variance.
#'
#'
#' Required features: `contrib`
#'
#' [make_consistent_b_ary_tree in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_consistent_b_ary_tree.html)
#'
#' **Citations:**
#'
#' * [HRMS09 Boosting the Accuracy of Differentially Private Histograms Through Consistency, section 4.1](https://arxiv.org/pdf/0904.0942.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Type:     `Vec<TIA>`
#' * Output Type:    `Vec<TOA>`
#'
#' @concept transformations
#' @param branching_factor the maximum number of children
#' @param .TIA Atomic type of the input data. Should be an integer type.
#' @param .TOA Atomic type of the output data. Should be a float type.
#' @return Function
#' @export
make_consistent_b_ary_tree <- function(
  branching_factor,
  .TIA = "int",
  .TOA = "float"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- rt_parse(type_name = .TIA, generics = list(".TOA"))
  .TOA <- rt_parse(type_name = .TOA, generics = list(".TIA"))
  .TIA <- rt_substitute(.TIA, .TOA = .TOA)
  .TOA <- rt_substitute(.TOA, .TIA = .TIA)

  log_ <- new_constructor_log("make_consistent_b_ary_tree", "transformations", new_hashtab(
    list("branching_factor", "TIA", "TOA"),
    list(unbox2(branching_factor), .TIA, .TOA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = u32, inferred = rt_infer(branching_factor))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_consistent_b_ary_tree",
    branching_factor, .TIA, .TOA,
    log_, PACKAGE = "opendp")
  output
}

#' partial consistent b ary tree constructor
#'
#' See documentation for [make_consistent_b_ary_tree()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param branching_factor the maximum number of children
#' @param .TIA Atomic type of the input data. Should be an integer type.
#' @param .TOA Atomic type of the output data. Should be a float type.
#' @return Function
#' @export
then_consistent_b_ary_tree <- function(
  lhs,
  branching_factor,
  .TIA = "int",
  .TOA = "float"
) {

  log_ <- new_constructor_log("then_consistent_b_ary_tree", "transformations", new_hashtab(
    list("branching_factor", "TIA", "TOA"),
    list(unbox2(branching_factor), .TIA, .TOA)
  ))

  make_chain_dyn(
    make_consistent_b_ary_tree(
      branching_factor = branching_factor,
      .TIA = .TIA,
      .TOA = .TOA),
    lhs,
    log_)
}


#' count constructor
#'
#' Make a Transformation that computes a count of the number of records in data.
#'
#'
#' Required features: `contrib`
#'
#' [make_count in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_count.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<TO>`
#' * Output Metric:  `AbsoluteDistance<TO>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/count/make_count.pdf)
#'
#' @concept transformations
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param .TO Output Type. Must be numeric.
#' @return Transformation
#' @export
make_count <- function(
  input_domain,
  input_metric,
  .TO = "int"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TO <- rt_parse(type_name = .TO)

  log_ <- new_constructor_log("make_count", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TO"),
    list(input_domain, input_metric, .TO)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_count",
    input_domain, input_metric, .TO,
    log_, PACKAGE = "opendp")
  output
}

#' partial count constructor
#'
#' See documentation for [make_count()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TO Output Type. Must be numeric.
#' @return Transformation
#' @export
then_count <- function(
  lhs,
  .TO = "int"
) {

  log_ <- new_constructor_log("then_count", "transformations", new_hashtab(
    list("TO"),
    list(.TO)
  ))

  make_chain_dyn(
    make_count(
      output_domain(lhs),
      output_metric(lhs),
      .TO = .TO),
    lhs,
    log_)
}


#' count by constructor
#'
#' Make a Transformation that computes the count of each unique value in data.
#' This assumes that the category set is unknown.
#'
#'
#' Required features: `contrib`
#'
#' [make_count_by in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_count_by.html)
#'
#' **Citations:**
#'
#' * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TK>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
#' * Output Metric:  `L01InfDistance<AbsoluteDistance<TV>>`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .TV Type of Value. Express counts in terms of this integral type.
#' @return The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
#' @export
make_count_by <- function(
  input_domain,
  input_metric,
  .TV = "int"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TV <- rt_parse(type_name = .TV)

  log_ <- new_constructor_log("make_count_by", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TV"),
    list(input_domain, input_metric, .TV)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_count_by",
    input_domain, input_metric, .TV,
    log_, PACKAGE = "opendp")
  output
}

#' partial count by constructor
#'
#' See documentation for [make_count_by()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TV Type of Value. Express counts in terms of this integral type.
#' @return The carrier type is `HashMap<TK, TV>`, a hashmap of the count (`TV`) for each unique data input (`TK`).
#' @export
then_count_by <- function(
  lhs,
  .TV = "int"
) {

  log_ <- new_constructor_log("then_count_by", "transformations", new_hashtab(
    list("TV"),
    list(.TV)
  ))

  make_chain_dyn(
    make_count_by(
      output_domain(lhs),
      output_metric(lhs),
      .TV = .TV),
    lhs,
    log_)
}


#' count by categories constructor
#'
#' Make a Transformation that computes the number of times each category appears in the data.
#' This assumes that the category set is known.
#'
#'
#' Required features: `contrib`
#'
#' [make_count_by_categories in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_count_by_categories.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#' * [BV17 Differential Privacy on Finite Computers](https://arxiv.org/abs/1709.05396)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
#' * Output Metric:  `MO`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param categories The set of categories to compute counts for.
#' @param null_category Include a count of the number of elements that were not in the category set at the end of the vector.
#' @param .MO Output Metric.
#' @param .TOA Atomic Output Type that is numeric.
#' @return The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).
#' @export
make_count_by_categories <- function(
  input_domain,
  input_metric,
  categories,
  null_category = TRUE,
  .MO = "L1Distance<int>",
  .TOA = "int"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO, generics = list(".TOA", ".TIA"))
  .TOA <- rt_parse(type_name = .TOA, generics = list(".MO", ".TIA"))
  .TIA <- get_atom(get_type(input_domain))
  .MO <- rt_substitute(.MO, .TOA = .TOA, .TIA = .TIA)
  .TOA <- rt_substitute(.TOA, .MO = .MO, .TIA = .TIA)
  .T.categories <- new_runtime_type(origin = "Vec", args = list(.TIA))

  log_ <- new_constructor_log("make_count_by_categories", "transformations", new_hashtab(
    list("input_domain", "input_metric", "categories", "null_category", "MO", "TOA"),
    list(input_domain, input_metric, categories, unbox2(null_category), .MO, .TOA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.categories, inferred = rt_infer(categories))
  rt_assert_is_similar(expected = bool, inferred = rt_infer(null_category))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_count_by_categories",
    input_domain, input_metric, categories, null_category, .MO, .TOA, .TIA, rt_parse(.T.categories),
    log_, PACKAGE = "opendp")
  output
}

#' partial count by categories constructor
#'
#' See documentation for [make_count_by_categories()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param categories The set of categories to compute counts for.
#' @param null_category Include a count of the number of elements that were not in the category set at the end of the vector.
#' @param .MO Output Metric.
#' @param .TOA Atomic Output Type that is numeric.
#' @return The carrier type is `Vec<TOA>`, a vector of the counts (`TOA`).
#' @export
then_count_by_categories <- function(
  lhs,
  categories,
  null_category = TRUE,
  .MO = "L1Distance<int>",
  .TOA = "int"
) {

  log_ <- new_constructor_log("then_count_by_categories", "transformations", new_hashtab(
    list("categories", "null_category", "MO", "TOA"),
    list(categories, unbox2(null_category), .MO, .TOA)
  ))

  make_chain_dyn(
    make_count_by_categories(
      output_domain(lhs),
      output_metric(lhs),
      categories = categories,
      null_category = null_category,
      .MO = .MO,
      .TOA = .TOA),
    lhs,
    log_)
}


#' count distinct constructor
#'
#' Make a Transformation that computes a count of the number of unique, distinct records in data.
#'
#'
#' Required features: `contrib`
#'
#' [make_count_distinct in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_count_distinct.html)
#'
#' **Citations:**
#'
#' * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<TO>`
#' * Output Metric:  `AbsoluteDistance<TO>`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .TO Output Type. Must be numeric.
#' @return Transformation
#' @export
make_count_distinct <- function(
  input_domain,
  input_metric,
  .TO = "int"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TO <- rt_parse(type_name = .TO)

  log_ <- new_constructor_log("make_count_distinct", "transformations", new_hashtab(
    list("input_domain", "input_metric", "TO"),
    list(input_domain, input_metric, .TO)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_count_distinct",
    input_domain, input_metric, .TO,
    log_, PACKAGE = "opendp")
  output
}

#' partial count distinct constructor
#'
#' See documentation for [make_count_distinct()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .TO Output Type. Must be numeric.
#' @return Transformation
#' @export
then_count_distinct <- function(
  lhs,
  .TO = "int"
) {

  log_ <- new_constructor_log("then_count_distinct", "transformations", new_hashtab(
    list("TO"),
    list(.TO)
  ))

  make_chain_dyn(
    make_count_distinct(
      output_domain(lhs),
      output_metric(lhs),
      .TO = .TO),
    lhs,
    log_)
}


#' create dataframe constructor
#'
#' Make a Transformation that constructs a dataframe from a `Vec<Vec<String>>` (a vector of records).
#'
#'
#' Required features: `contrib`
#'
#' [make_create_dataframe in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_create_dataframe.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<VectorDomain<AtomDomain<String>>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `DataFrameDomain<K>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#' @param col_names Column names for each record entry.
#' @param .K categorical/hashable data type of column names
#' @return Transformation
#' @export
make_create_dataframe <- function(
  col_names,
  .K = NULL
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .K <- parse_or_infer(type_name = .K, public_example = get_first(col_names))
  .T.col_names <- new_runtime_type(origin = "Vec", args = list(.K))

  log_ <- new_constructor_log("make_create_dataframe", "transformations", new_hashtab(
    list("col_names", "K"),
    list(col_names, .K)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.col_names, inferred = rt_infer(col_names))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_create_dataframe",
    col_names, .K, rt_parse(.T.col_names),
    log_, PACKAGE = "opendp")
  output
}

#' partial create dataframe constructor
#'
#' See documentation for [make_create_dataframe()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param col_names Column names for each record entry.
#' @param .K categorical/hashable data type of column names
#' @return Transformation
#' @export
then_create_dataframe <- function(
  lhs,
  col_names,
  .K = NULL
) {

  log_ <- new_constructor_log("then_create_dataframe", "transformations", new_hashtab(
    list("col_names", "K"),
    list(col_names, .K)
  ))

  make_chain_dyn(
    make_create_dataframe(
      col_names = col_names,
      .K = .K),
    lhs,
    log_)
}


#' df cast default constructor
#'
#' Make a Transformation that casts the elements in a column in a dataframe from type `TIA` to type `TOA`.
#' If cast fails, fill with default.
#'
#'
#' | `TIA`  | `TIA::default()` |
#' | ------ | ---------------- |
#' | float  | `0.`             |
#' | int    | `0`              |
#' | string | `""`             |
#' | bool   | `false`          |
#'
#'
#' Required features: `contrib`
#'
#' [make_df_cast_default in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_df_cast_default.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DataFrameDomain<TK>`
#' * Output Domain:  `M`
#' * Input Metric:   `DataFrameDomain<TK>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param column_name column name to be transformed
#' @param .TIA Atomic Input Type to cast from
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
make_df_cast_default <- function(
  input_domain,
  input_metric,
  column_name,
  .TIA,
  .TOA
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- rt_parse(type_name = .TIA, generics = list(".TOA", ".TK", ".M"))
  .TOA <- rt_parse(type_name = .TOA, generics = list(".TIA", ".TK", ".M"))
  .TK <- get_atom(get_type(input_domain))
  .M <- get_type(input_metric)
  .TIA <- rt_substitute(.TIA, .TOA = .TOA, .TK = .TK, .M = .M)
  .TOA <- rt_substitute(.TOA, .TIA = .TIA, .TK = .TK, .M = .M)

  log_ <- new_constructor_log("make_df_cast_default", "transformations", new_hashtab(
    list("input_domain", "input_metric", "column_name", "TIA", "TOA"),
    list(input_domain, input_metric, column_name, .TIA, .TOA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TK, inferred = rt_infer(column_name))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_df_cast_default",
    input_domain, input_metric, column_name, .TIA, .TOA, .TK, .M,
    log_, PACKAGE = "opendp")
  output
}

#' partial df cast default constructor
#'
#' See documentation for [make_df_cast_default()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param column_name column name to be transformed
#' @param .TIA Atomic Input Type to cast from
#' @param .TOA Atomic Output Type to cast into
#' @return Transformation
#' @export
then_df_cast_default <- function(
  lhs,
  column_name,
  .TIA,
  .TOA
) {

  log_ <- new_constructor_log("then_df_cast_default", "transformations", new_hashtab(
    list("column_name", "TIA", "TOA"),
    list(column_name, .TIA, .TOA)
  ))

  make_chain_dyn(
    make_df_cast_default(
      output_domain(lhs),
      output_metric(lhs),
      column_name = column_name,
      .TIA = .TIA,
      .TOA = .TOA),
    lhs,
    log_)
}


#' df is equal constructor
#'
#' Make a Transformation that checks if each element in a column in a dataframe is equivalent to `value`.
#'
#'
#' Required features: `contrib`
#'
#' [make_df_is_equal in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_df_is_equal.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DataFrameDomain<TK>`
#' * Output Domain:  `M`
#' * Input Metric:   `DataFrameDomain<TK>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param column_name Column name to be transformed
#' @param value Value to check for equality
#' @param .TIA Atomic Input Type to cast from
#' @return Transformation
#' @export
make_df_is_equal <- function(
  input_domain,
  input_metric,
  column_name,
  value,
  .TIA = NULL
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- parse_or_infer(type_name = .TIA, public_example = value, generics = list(".TK", ".M"))
  .TK <- get_atom(get_type(input_domain))
  .M <- get_type(input_metric)
  .TIA <- rt_substitute(.TIA, .TK = .TK, .M = .M)

  log_ <- new_constructor_log("make_df_is_equal", "transformations", new_hashtab(
    list("input_domain", "input_metric", "column_name", "value", "TIA"),
    list(input_domain, input_metric, column_name, value, .TIA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TK, inferred = rt_infer(column_name))
  rt_assert_is_similar(expected = .TIA, inferred = rt_infer(value))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_df_is_equal",
    input_domain, input_metric, column_name, value, .TIA, .TK, .M,
    log_, PACKAGE = "opendp")
  output
}

#' partial df is equal constructor
#'
#' See documentation for [make_df_is_equal()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param column_name Column name to be transformed
#' @param value Value to check for equality
#' @param .TIA Atomic Input Type to cast from
#' @return Transformation
#' @export
then_df_is_equal <- function(
  lhs,
  column_name,
  value,
  .TIA = NULL
) {

  log_ <- new_constructor_log("then_df_is_equal", "transformations", new_hashtab(
    list("column_name", "value", "TIA"),
    list(column_name, value, .TIA)
  ))

  make_chain_dyn(
    make_df_is_equal(
      output_domain(lhs),
      output_metric(lhs),
      column_name = column_name,
      value = value,
      .TIA = .TIA),
    lhs,
    log_)
}


#' drop null constructor
#'
#' Make a Transformation that drops null values.
#'
#'
#' | input_domain                                    |
#' | ----------------------------------------------- |
#' | `vector_domain(option_domain(atom_domain(TA)))` |
#' | `vector_domain(atom_domain(TA))`                |
#'
#'
#' Required features: `contrib`
#'
#' [make_drop_null in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_drop_null.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<DIA>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<DIA::Imputed>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_drop_null <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_drop_null", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_drop_null",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial drop null constructor
#'
#' See documentation for [make_drop_null()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_drop_null <- function(
  lhs
) {

  log_ <- new_constructor_log("then_drop_null", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_drop_null(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' find constructor
#'
#' Find the index of a data value in a set of categories.
#'
#' For each value in the input vector, finds the index of the value in `categories`.
#' If an index is found, returns `Some(index)`, else `None`.
#' Chain with `make_impute_constant` or `make_drop_null` to handle nullity.
#'
#'
#' Required features: `contrib`
#'
#' [make_find in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_find.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<OptionDomain<AtomDomain<usize>>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain The domain of the input vector.
#' @param input_metric The metric of the input vector.
#' @param categories The set of categories to find indexes from.
#' @return Transformation
#' @export
make_find <- function(
  input_domain,
  input_metric,
  categories
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- get_atom(get_type(input_domain))
  .T.categories <- new_runtime_type(origin = "Vec", args = list(.TIA))

  log_ <- new_constructor_log("make_find", "transformations", new_hashtab(
    list("input_domain", "input_metric", "categories"),
    list(input_domain, input_metric, categories)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.categories, inferred = rt_infer(categories))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_find",
    input_domain, input_metric, categories, .TIA, rt_parse(.T.categories),
    log_, PACKAGE = "opendp")
  output
}

#' partial find constructor
#'
#' See documentation for [make_find()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param categories The set of categories to find indexes from.
#' @return Transformation
#' @export
then_find <- function(
  lhs,
  categories
) {

  log_ <- new_constructor_log("then_find", "transformations", new_hashtab(
    list("categories"),
    list(categories)
  ))

  make_chain_dyn(
    make_find(
      output_domain(lhs),
      output_metric(lhs),
      categories = categories),
    lhs,
    log_)
}


#' find bin constructor
#'
#' Make a transformation that finds the bin index in a monotonically increasing vector of edges.
#'
#' For each value in the input vector, finds the index of the bin the value falls into.
#' `edges` splits the entire range of `TIA` into bins.
#' The first bin at index zero ranges from negative infinity to the first edge, non-inclusive.
#' The last bin at index `edges.len()` ranges from the last bin, inclusive, to positive infinity.
#'
#' To be valid, `edges` must be unique and ordered.
#' `edges` are left inclusive, right exclusive.
#'
#'
#' Required features: `contrib`
#'
#' [make_find_bin in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_find_bin.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<usize>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain The domain of the input vector.
#' @param input_metric The metric of the input vector.
#' @param edges The set of edges to split bins by.
#' @return Transformation
#' @export
make_find_bin <- function(
  input_domain,
  input_metric,
  edges
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- get_atom(get_type(input_domain))
  .T.edges <- new_runtime_type(origin = "Vec", args = list(.TIA))

  log_ <- new_constructor_log("make_find_bin", "transformations", new_hashtab(
    list("input_domain", "input_metric", "edges"),
    list(input_domain, input_metric, edges)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.edges, inferred = rt_infer(edges))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_find_bin",
    input_domain, input_metric, edges, .TIA, rt_parse(.T.edges),
    log_, PACKAGE = "opendp")
  output
}

#' partial find bin constructor
#'
#' See documentation for [make_find_bin()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param edges The set of edges to split bins by.
#' @return Transformation
#' @export
then_find_bin <- function(
  lhs,
  edges
) {

  log_ <- new_constructor_log("then_find_bin", "transformations", new_hashtab(
    list("edges"),
    list(edges)
  ))

  make_chain_dyn(
    make_find_bin(
      output_domain(lhs),
      output_metric(lhs),
      edges = edges),
    lhs,
    log_)
}


#' identity constructor
#'
#' Make a Transformation representing the identity function.
#'
#' WARNING: In Python, this function does not ensure that the domain and metric form a valid metric space.
#' However, if the domain and metric do not form a valid metric space,
#' then the resulting Transformation won't be chainable with any valid Transformation,
#' so it cannot be used to introduce an invalid metric space into a chain of valid Transformations.
#'
#'
#' Required features: `contrib`, `honest-but-curious`
#'
#' [make_identity in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_identity.html)
#'
#' **Why honest-but-curious?:**
#'
#' For the result to be a valid transformation, the `input_domain` and `input_metric` pairing must form a valid metric space.
#' For instance, the symmetric distance metric and atom domain do not form a valid metric space,
#' because the metric cannot be used to measure distances between any two elements of an atom domain.
#' Whereas, the symmetric distance metric and vector domain,
#' or absolute distance metric and atom domain on a scalar type, both form valid metric spaces.
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Domain:  `M`
#' * Input Metric:   `D`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param domain Domain of input data
#' @param metric Metric on input domain
#' @return Transformation
#' @export
make_identity <- function(
  domain,
  metric
) {
  assert_features("contrib", "honest-but-curious")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_identity", "transformations", new_hashtab(
    list("domain", "metric"),
    list(domain, metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_identity",
    domain, metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial identity constructor
#'
#' See documentation for [make_identity()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_identity <- function(
  lhs
) {

  log_ <- new_constructor_log("then_identity", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_identity(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' impute constant constructor
#'
#' Make a Transformation that replaces null/None data with `constant`.
#'
#' If chaining after a `make_cast`, the input type is `Option<Vec<TA>>`.
#' If chaining after a `make_cast_inherent`, the input type is `Vec<TA>`, where `TA` may take on float NaNs.
#'
#' | input_domain                                    |  Input Data Type  |
#' | ----------------------------------------------- | ----------------- |
#' | `vector_domain(option_domain(atom_domain(TA)))` | `Vec<Option<TA>>` |
#' | `vector_domain(atom_domain(TA))`                | `Vec<TA>`         |
#'
#'
#' Required features: `contrib`
#'
#' [make_impute_constant in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_impute_constant.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<DIA>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<DIA::Imputed>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of the input data. See table above.
#' @param input_metric Metric of the input data. A dataset metric.
#' @param constant Value to replace nulls with.
#' @return Transformation
#' @export
make_impute_constant <- function(
  input_domain,
  input_metric,
  constant
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T.constant <- get_atom(get_type(input_domain))

  log_ <- new_constructor_log("make_impute_constant", "transformations", new_hashtab(
    list("input_domain", "input_metric", "constant"),
    list(input_domain, input_metric, constant)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.constant, inferred = rt_infer(constant))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_impute_constant",
    input_domain, input_metric, constant, rt_parse(.T.constant),
    log_, PACKAGE = "opendp")
  output
}

#' partial impute constant constructor
#'
#' See documentation for [make_impute_constant()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param constant Value to replace nulls with.
#' @return Transformation
#' @export
then_impute_constant <- function(
  lhs,
  constant
) {

  log_ <- new_constructor_log("then_impute_constant", "transformations", new_hashtab(
    list("constant"),
    list(constant)
  ))

  make_chain_dyn(
    make_impute_constant(
      output_domain(lhs),
      output_metric(lhs),
      constant = constant),
    lhs,
    log_)
}


#' impute uniform float constructor
#'
#' Make a Transformation that replaces NaN values in `Vec<TA>` with uniformly distributed floats within `bounds`.
#'
#'
#' Required features: `contrib`
#'
#' [make_impute_uniform_float in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_impute_uniform_float.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TA>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of the input.
#' @param input_metric Metric of the input.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
make_impute_uniform_float <- function(
  input_domain,
  input_metric,
  bounds
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TA <- get_atom(get_type(input_domain))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.TA, .TA))

  log_ <- new_constructor_log("make_impute_uniform_float", "transformations", new_hashtab(
    list("input_domain", "input_metric", "bounds"),
    list(input_domain, input_metric, lapply(bounds, unbox2))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_impute_uniform_float",
    input_domain, input_metric, bounds, .TA, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial impute uniform float constructor
#'
#' See documentation for [make_impute_uniform_float()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
then_impute_uniform_float <- function(
  lhs,
  bounds
) {

  log_ <- new_constructor_log("then_impute_uniform_float", "transformations", new_hashtab(
    list("bounds"),
    list(lapply(bounds, unbox2))
  ))

  make_chain_dyn(
    make_impute_uniform_float(
      output_domain(lhs),
      output_metric(lhs),
      bounds = bounds),
    lhs,
    log_)
}


#' index constructor
#'
#' Make a transformation that treats each element as an index into a vector of categories.
#'
#'
#' Required features: `contrib`
#'
#' [make_index in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_index.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<usize>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain The domain of the input vector.
#' @param input_metric The metric of the input vector.
#' @param categories The set of categories to index into.
#' @param null Category to return if the index is out-of-range of the category set.
#' @param .TOA Atomic Output Type. Output data will be `Vec<TOA>`.
#' @return Transformation
#' @export
make_index <- function(
  input_domain,
  input_metric,
  categories,
  null,
  .TOA = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TOA <- parse_or_infer(type_name = .TOA, public_example = get_first(categories))
  .T.categories <- new_runtime_type(origin = "Vec", args = list(.TOA))

  log_ <- new_constructor_log("make_index", "transformations", new_hashtab(
    list("input_domain", "input_metric", "categories", "null", "TOA"),
    list(input_domain, input_metric, categories, null, .TOA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.categories, inferred = rt_infer(categories))
  rt_assert_is_similar(expected = .TOA, inferred = rt_infer(null))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_index",
    input_domain, input_metric, categories, null, .TOA, rt_parse(.T.categories),
    log_, PACKAGE = "opendp")
  output
}

#' partial index constructor
#'
#' See documentation for [make_index()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param categories The set of categories to index into.
#' @param null Category to return if the index is out-of-range of the category set.
#' @param .TOA Atomic Output Type. Output data will be `Vec<TOA>`.
#' @return Transformation
#' @export
then_index <- function(
  lhs,
  categories,
  null,
  .TOA = NULL
) {

  log_ <- new_constructor_log("then_index", "transformations", new_hashtab(
    list("categories", "null", "TOA"),
    list(categories, null, .TOA)
  ))

  make_chain_dyn(
    make_index(
      output_domain(lhs),
      output_metric(lhs),
      categories = categories,
      null = null,
      .TOA = .TOA),
    lhs,
    log_)
}


#' is equal constructor
#'
#' Make a Transformation that checks if each element is equal to `value`.
#'
#'
#' Required features: `contrib`
#'
#' [make_is_equal in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_is_equal.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<bool>>`
#' * Output Metric:  `M`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/manipulation/make_is_equal.pdf)
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param value value to check against
#' @return Transformation
#' @export
make_is_equal <- function(
  input_domain,
  input_metric,
  value
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- get_atom(get_type(input_domain))
  .M <- get_type(input_metric)

  log_ <- new_constructor_log("make_is_equal", "transformations", new_hashtab(
    list("input_domain", "input_metric", "value"),
    list(input_domain, input_metric, value)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TIA, inferred = rt_infer(value))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_is_equal",
    input_domain, input_metric, value, .TIA, .M,
    log_, PACKAGE = "opendp")
  output
}

#' partial is equal constructor
#'
#' See documentation for [make_is_equal()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param value value to check against
#' @return Transformation
#' @export
then_is_equal <- function(
  lhs,
  value
) {

  log_ <- new_constructor_log("then_is_equal", "transformations", new_hashtab(
    list("value"),
    list(value)
  ))

  make_chain_dyn(
    make_is_equal(
      output_domain(lhs),
      output_metric(lhs),
      value = value),
    lhs,
    log_)
}


#' is null constructor
#'
#' Make a Transformation that checks if each element in a vector is null or nan.
#'
#'
#' Required features: `contrib`
#'
#' [make_is_null in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_is_null.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<DIA>`
#' * Output Domain:  `M`
#' * Input Metric:   `VectorDomain<AtomDomain<bool>>`
#' * Output Metric:  `M`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_is_null <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_is_null", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_is_null",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial is null constructor
#'
#' See documentation for [make_is_null()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_is_null <- function(
  lhs
) {

  log_ <- new_constructor_log("then_is_null", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_is_null(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' lipschitz float mul constructor
#'
#' Make a transformation that multiplies an aggregate by a constant.
#'
#' The bounds clamp the input, in order to bound the increase in sensitivity from float rounding.
#'
#'
#' Required features: `contrib`
#'
#' [make_lipschitz_float_mul in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_lipschitz_float_mul.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<TA>`
#' * Output Domain:  `AbsoluteDistance<TA>`
#' * Input Metric:   `AtomDomain<TA>`
#' * Output Metric:  `AbsoluteDistance<TA>`
#'
#' @concept transformations
#' @param input_domain The domain of the input.
#' @param input_metric The metric of the input.
#' @param constant The constant to multiply aggregates by.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
make_lipschitz_float_mul <- function(
  input_domain,
  input_metric,
  constant,
  bounds
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TA <- get_atom(get_carrier_type(input_domain))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.TA, .TA))

  log_ <- new_constructor_log("make_lipschitz_float_mul", "transformations", new_hashtab(
    list("input_domain", "input_metric", "constant", "bounds"),
    list(input_domain, input_metric, constant, lapply(bounds, unbox2))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TA, inferred = rt_infer(constant))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_lipschitz_float_mul",
    input_domain, input_metric, constant, bounds, .TA, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial lipschitz float mul constructor
#'
#' See documentation for [make_lipschitz_float_mul()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param constant The constant to multiply aggregates by.
#' @param bounds Tuple of inclusive lower and upper bounds.
#' @return Transformation
#' @export
then_lipschitz_float_mul <- function(
  lhs,
  constant,
  bounds
) {

  log_ <- new_constructor_log("then_lipschitz_float_mul", "transformations", new_hashtab(
    list("constant", "bounds"),
    list(constant, lapply(bounds, unbox2))
  ))

  make_chain_dyn(
    make_lipschitz_float_mul(
      output_domain(lhs),
      output_metric(lhs),
      constant = constant,
      bounds = bounds),
    lhs,
    log_)
}


#' mean constructor
#'
#' Make a Transformation that computes the mean of bounded data.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size.
#' Use `make_clamp` to bound data and `make_resize` to establish dataset size.
#'
#'
#' Required features: `contrib`
#'
#' [make_mean in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_mean.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `MI`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_mean <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_mean", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_mean",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial mean constructor
#'
#' See documentation for [make_mean()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_mean <- function(
  lhs
) {

  log_ <- new_constructor_log("then_mean", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_mean(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' metric bounded constructor
#'
#' Make a Transformation that converts the unbounded dataset metric `MI`
#' to the respective bounded dataset metric with a no-op.
#'
#' The constructor enforces that the input domain has known size,
#' because it must have known size to be valid under a bounded dataset metric.
#'
#' | `MI`                 | `MI::BoundedMetric` |
#' | -------------------- | ------------------- |
#' | SymmetricDistance    | ChangeOneDistance   |
#' | InsertDeleteDistance | HammingDistance     |
#'
#'
#' Required features: `contrib`
#'
#' [make_metric_bounded in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_metric_bounded.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Domain:  `MI`
#' * Input Metric:   `D`
#' * Output Metric:  `MI::BoundedMetric`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_metric_bounded <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_metric_bounded", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_metric_bounded",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial metric bounded constructor
#'
#' See documentation for [make_metric_bounded()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_metric_bounded <- function(
  lhs
) {

  log_ <- new_constructor_log("then_metric_bounded", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_metric_bounded(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' metric unbounded constructor
#'
#' Make a Transformation that converts the bounded dataset metric `MI`
#' to the respective unbounded dataset metric with a no-op.
#'
#' | `MI`              | `MI::UnboundedMetric` |
#' | ----------------- | --------------------- |
#' | ChangeOneDistance | SymmetricDistance     |
#' | HammingDistance   | InsertDeleteDistance  |
#'
#'
#' Required features: `contrib`
#'
#' [make_metric_unbounded in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_metric_unbounded.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Domain:  `MI`
#' * Input Metric:   `D`
#' * Output Metric:  `MI::UnboundedMetric`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_metric_unbounded <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_metric_unbounded", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_metric_unbounded",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial metric unbounded constructor
#'
#' See documentation for [make_metric_unbounded()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_metric_unbounded <- function(
  lhs
) {

  log_ <- new_constructor_log("then_metric_unbounded", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_metric_unbounded(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' ordered random constructor
#'
#' Make a Transformation that converts the unordered dataset metric `SymmetricDistance`
#' to the respective ordered dataset metric `InsertDeleteDistance` by assigning a random permutation.
#'
#' | `MI`              | `MI::OrderedMetric`  |
#' | ----------------- | -------------------- |
#' | SymmetricDistance | InsertDeleteDistance |
#' | ChangeOneDistance | HammingDistance      |
#'
#'
#' Required features: `contrib`
#'
#' [make_ordered_random in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_ordered_random.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Domain:  `MI`
#' * Input Metric:   `D`
#' * Output Metric:  `MI::OrderedMetric`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_ordered_random <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_ordered_random", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_ordered_random",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial ordered random constructor
#'
#' See documentation for [make_ordered_random()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_ordered_random <- function(
  lhs
) {

  log_ <- new_constructor_log("then_ordered_random", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_ordered_random(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' quantile score candidates constructor
#'
#' Makes a Transformation that scores how similar each candidate is to the given `alpha`-quantile on the input dataset.
#'
#'
#' Required features: `contrib`
#'
#' [make_quantile_score_candidates in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_quantile_score_candidates.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Domain:  `MI`
#' * Input Metric:   `VectorDomain<AtomDomain<u64>>`
#' * Output Metric:  `LInfDistance<u64>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/transformations/quantile_score_candidates/make_quantile_score_candidates.pdf)
#'
#' @concept transformations
#' @param input_domain Uses a smaller sensitivity when the size of vectors in the input domain is known.
#' @param input_metric Either SymmetricDistance or InsertDeleteDistance.
#' @param candidates Potential quantiles to score
#' @param alpha a value in \eqn{[0, 1]}. Choose 0.5 for median
#' @return Transformation
#' @export
make_quantile_score_candidates <- function(
  input_domain,
  input_metric,
  candidates,
  alpha
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TIA <- get_atom(get_type(input_domain))
  .T.candidates <- new_runtime_type(origin = "Vec", args = list(.TIA))

  log_ <- new_constructor_log("make_quantile_score_candidates", "transformations", new_hashtab(
    list("input_domain", "input_metric", "candidates", "alpha"),
    list(input_domain, input_metric, candidates, unbox2(alpha))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.candidates, inferred = rt_infer(candidates))
  rt_assert_is_similar(expected = f64, inferred = rt_infer(alpha))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_quantile_score_candidates",
    input_domain, input_metric, candidates, alpha, .TIA, rt_parse(.T.candidates),
    log_, PACKAGE = "opendp")
  output
}

#' partial quantile score candidates constructor
#'
#' See documentation for [make_quantile_score_candidates()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param candidates Potential quantiles to score
#' @param alpha a value in \eqn{[0, 1]}. Choose 0.5 for median
#' @return Transformation
#' @export
then_quantile_score_candidates <- function(
  lhs,
  candidates,
  alpha
) {

  log_ <- new_constructor_log("then_quantile_score_candidates", "transformations", new_hashtab(
    list("candidates", "alpha"),
    list(candidates, unbox2(alpha))
  ))

  make_chain_dyn(
    make_quantile_score_candidates(
      output_domain(lhs),
      output_metric(lhs),
      candidates = candidates,
      alpha = alpha),
    lhs,
    log_)
}


#' quantiles from counts constructor
#'
#' Postprocess a noisy array of summary counts into quantiles.
#'
#'
#' Required features: `contrib`
#'
#' [make_quantiles_from_counts in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_quantiles_from_counts.html)
#'
#' **Supporting Elements:**
#'
#' * Input Type:     `Vec<TA>`
#' * Output Type:    `Vec<TA>`
#'
#' @concept transformations
#' @param bin_edges The edges that the input data was binned into before counting.
#' @param alphas Return all specified `alpha`-quantiles.
#' @param interpolation Must be one of `linear` or `nearest`
#' @param .TA Atomic Type of the bin edges and data.
#' @param .F Float type of the alpha argument. One of `f32` or `f64`
#' @return Function
#' @export
make_quantiles_from_counts <- function(
  bin_edges,
  alphas,
  interpolation = "linear",
  .TA = NULL,
  .F = "float"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .TA <- parse_or_infer(type_name = .TA, public_example = get_first(bin_edges), generics = list(".F"))
  .F <- parse_or_infer(type_name = .F, public_example = get_first(alphas), generics = list(".TA"))
  .TA <- rt_substitute(.TA, .F = .F)
  .F <- rt_substitute(.F, .TA = .TA)
  .T.bin_edges <- new_runtime_type(origin = "Vec", args = list(.TA))
  .T.alphas <- new_runtime_type(origin = "Vec", args = list(.F))

  log_ <- new_constructor_log("make_quantiles_from_counts", "transformations", new_hashtab(
    list("bin_edges", "alphas", "interpolation", "TA", "F"),
    list(bin_edges, alphas, unbox2(interpolation), .TA, .F)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.bin_edges, inferred = rt_infer(bin_edges))
  rt_assert_is_similar(expected = .T.alphas, inferred = rt_infer(alphas))
  rt_assert_is_similar(expected = String, inferred = rt_infer(interpolation))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_quantiles_from_counts",
    bin_edges, alphas, interpolation, .TA, .F, rt_parse(.T.bin_edges), rt_parse(.T.alphas),
    log_, PACKAGE = "opendp")
  output
}

#' partial quantiles from counts constructor
#'
#' See documentation for [make_quantiles_from_counts()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param bin_edges The edges that the input data was binned into before counting.
#' @param alphas Return all specified `alpha`-quantiles.
#' @param interpolation Must be one of `linear` or `nearest`
#' @param .TA Atomic Type of the bin edges and data.
#' @param .F Float type of the alpha argument. One of `f32` or `f64`
#' @return Function
#' @export
then_quantiles_from_counts <- function(
  lhs,
  bin_edges,
  alphas,
  interpolation = "linear",
  .TA = NULL,
  .F = "float"
) {

  log_ <- new_constructor_log("then_quantiles_from_counts", "transformations", new_hashtab(
    list("bin_edges", "alphas", "interpolation", "TA", "F"),
    list(bin_edges, alphas, unbox2(interpolation), .TA, .F)
  ))

  make_chain_dyn(
    make_quantiles_from_counts(
      bin_edges = bin_edges,
      alphas = alphas,
      interpolation = interpolation,
      .TA = .TA,
      .F = .F),
    lhs,
    log_)
}


#' resize constructor
#'
#' Make a Transformation that either truncates or imputes records
#' with `constant` to match a provided `size`.
#'
#'
#' Required features: `contrib`
#'
#' [make_resize in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_resize.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TA>>`
#' * Output Domain:  `MI`
#' * Input Metric:   `VectorDomain<AtomDomain<TA>>`
#' * Output Metric:  `MO`
#'
#' @concept transformations
#' @param input_domain Domain of input data.
#' @param input_metric Metric of input data.
#' @param size Number of records in output data.
#' @param constant Value to impute with.
#' @param .MO Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
#' @return A vector of the same type `TA`, but with the provided `size`.
#' @export
make_resize <- function(
  input_domain,
  input_metric,
  size,
  constant,
  .MO = "SymmetricDistance"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .MO <- rt_parse(type_name = .MO)
  .T.constant <- get_atom(get_type(input_domain))

  log_ <- new_constructor_log("make_resize", "transformations", new_hashtab(
    list("input_domain", "input_metric", "size", "constant", "MO"),
    list(input_domain, input_metric, unbox2(size), constant, .MO)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.constant, inferred = rt_infer(constant))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_resize",
    input_domain, input_metric, size, constant, .MO, rt_parse(.T.constant),
    log_, PACKAGE = "opendp")
  output
}

#' partial resize constructor
#'
#' See documentation for [make_resize()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in output data.
#' @param constant Value to impute with.
#' @param .MO Output Metric. One of `InsertDeleteDistance` or `SymmetricDistance`
#' @return A vector of the same type `TA`, but with the provided `size`.
#' @export
then_resize <- function(
  lhs,
  size,
  constant,
  .MO = "SymmetricDistance"
) {

  log_ <- new_constructor_log("then_resize", "transformations", new_hashtab(
    list("size", "constant", "MO"),
    list(unbox2(size), constant, .MO)
  ))

  make_chain_dyn(
    make_resize(
      output_domain(lhs),
      output_metric(lhs),
      size = size,
      constant = constant,
      .MO = .MO),
    lhs,
    log_)
}


#' select column constructor
#'
#' Make a Transformation that retrieves the column `key` from a dataframe as `Vec<TOA>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_select_column in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_select_column.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DataFrameDomain<K>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `VectorDomain<AtomDomain<TOA>>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#' @param key categorical/hashable data type of the key/column name
#' @param .K data type of key
#' @param .TOA Atomic Output Type to downcast vector to
#' @return Transformation
#' @export
make_select_column <- function(
  key,
  .TOA,
  .K = NULL
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .K <- parse_or_infer(type_name = .K, public_example = key, generics = list(".TOA"))
  .TOA <- rt_parse(type_name = .TOA, generics = list(".K"))
  .K <- rt_substitute(.K, .TOA = .TOA)
  .TOA <- rt_substitute(.TOA, .K = .K)

  log_ <- new_constructor_log("make_select_column", "transformations", new_hashtab(
    list("key", "K", "TOA"),
    list(key, .K, .TOA)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .K, inferred = rt_infer(key))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_select_column",
    key, .K, .TOA,
    log_, PACKAGE = "opendp")
  output
}

#' partial select column constructor
#'
#' See documentation for [make_select_column()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param key categorical/hashable data type of the key/column name
#' @param .K data type of key
#' @param .TOA Atomic Output Type to downcast vector to
#' @return Transformation
#' @export
then_select_column <- function(
  lhs,
  key,
  .TOA,
  .K = NULL
) {

  log_ <- new_constructor_log("then_select_column", "transformations", new_hashtab(
    list("key", "K", "TOA"),
    list(key, .K, .TOA)
  ))

  make_chain_dyn(
    make_select_column(
      key = key,
      .K = .K,
      .TOA = .TOA),
    lhs,
    log_)
}


#' sized bounded float checked sum constructor
#'
#' Make a Transformation that computes the sum of bounded floats with known dataset size.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#'
#' | S (summation algorithm) | input type     |
#' | ----------------------- | -------------- |
#' | `Sequential<S::Item>`   | `Vec<S::Item>` |
#' | `Pairwise<S::Item>`     | `Vec<S::Item>` |
#'
#' `S::Item` is the type of all of the following:
#' each bound, each element in the input data, the output data, and the output sensitivity.
#'
#' For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
#' set `S` to `Pairwise<f32>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_float_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_float_checked_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
make_sized_bounded_float_checked_sum <- function(
  size,
  bounds,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom_or_infer(.S, get_first(bounds))
  .S <- rt_substitute(.S, .T = .T)
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_float_checked_sum", "transformations", new_hashtab(
    list("size", "bounds", "S"),
    list(unbox2(size), lapply(bounds, unbox2), .S)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_float_checked_sum",
    size, bounds, .S, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded float checked sum constructor
#'
#' See documentation for [make_sized_bounded_float_checked_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
then_sized_bounded_float_checked_sum <- function(
  lhs,
  size,
  bounds,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_sized_bounded_float_checked_sum", "transformations", new_hashtab(
    list("size", "bounds", "S"),
    list(unbox2(size), lapply(bounds, unbox2), .S)
  ))

  make_chain_dyn(
    make_sized_bounded_float_checked_sum(
      size = size,
      bounds = bounds,
      .S = .S),
    lhs,
    log_)
}


#' sized bounded float ordered sum constructor
#'
#' Make a Transformation that computes the sum of bounded floats with known ordering and dataset size.
#'
#' Only useful when `make_bounded_float_checked_sum` returns an error due to potential for overflow.
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#' You may need to use `make_ordered_random` to impose an ordering on the data.
#'
#' | S (summation algorithm) | input type     |
#' | ----------------------- | -------------- |
#' | `Sequential<S::Item>`   | `Vec<S::Item>` |
#' | `Pairwise<S::Item>`     | `Vec<S::Item>` |
#'
#' `S::Item` is the type of all of the following:
#' each bound, each element in the input data, the output data, and the output sensitivity.
#'
#' For example, to construct a transformation that pairwise-sums `f32` half-precision floats,
#' set `S` to `Pairwise<f32>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_float_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_float_ordered_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `InsertDeleteDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
make_sized_bounded_float_ordered_sum <- function(
  size,
  bounds,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom_or_infer(.S, get_first(bounds))
  .S <- rt_substitute(.S, .T = .T)
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_float_ordered_sum", "transformations", new_hashtab(
    list("size", "bounds", "S"),
    list(unbox2(size), lapply(bounds, unbox2), .S)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_float_ordered_sum",
    size, bounds, .S, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded float ordered sum constructor
#'
#' See documentation for [make_sized_bounded_float_ordered_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .S Summation algorithm to use over some data type `T` (`T` is shorthand for `S::Item`)
#' @return Transformation
#' @export
then_sized_bounded_float_ordered_sum <- function(
  lhs,
  size,
  bounds,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_sized_bounded_float_ordered_sum", "transformations", new_hashtab(
    list("size", "bounds", "S"),
    list(unbox2(size), lapply(bounds, unbox2), .S)
  ))

  make_chain_dyn(
    make_sized_bounded_float_ordered_sum(
      size = size,
      bounds = bounds,
      .S = .S),
    lhs,
    log_)
}


#' sized bounded int checked sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints.
#' The effective range is reduced, as (bounds * size) must not overflow.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_int_checked_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_int_checked_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_sized_bounded_int_checked_sum <- function(
  size,
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_int_checked_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_int_checked_sum",
    size, bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded int checked sum constructor
#'
#' See documentation for [make_sized_bounded_int_checked_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_sized_bounded_int_checked_sum <- function(
  lhs,
  size,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_sized_bounded_int_checked_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_sized_bounded_int_checked_sum(
      size = size,
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' sized bounded int monotonic sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints,
#' where all values share the same sign.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_int_monotonic_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_int_monotonic_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_sized_bounded_int_monotonic_sum <- function(
  size,
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_int_monotonic_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_int_monotonic_sum",
    size, bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded int monotonic sum constructor
#'
#' See documentation for [make_sized_bounded_int_monotonic_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_sized_bounded_int_monotonic_sum <- function(
  lhs,
  size,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_sized_bounded_int_monotonic_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_sized_bounded_int_monotonic_sum(
      size = size,
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' sized bounded int ordered sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints with known dataset size.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#' You may need to use `make_ordered_random` to impose an ordering on the data.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_int_ordered_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_int_ordered_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `InsertDeleteDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_sized_bounded_int_ordered_sum <- function(
  size,
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_int_ordered_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_int_ordered_sum",
    size, bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded int ordered sum constructor
#'
#' See documentation for [make_sized_bounded_int_ordered_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_sized_bounded_int_ordered_sum <- function(
  lhs,
  size,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_sized_bounded_int_ordered_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_sized_bounded_int_ordered_sum(
      size = size,
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' sized bounded int split sum constructor
#'
#' Make a Transformation that computes the sum of bounded ints with known dataset size.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#' Adds the saturating sum of the positives to the saturating sum of the negatives.
#'
#'
#' Required features: `contrib`
#'
#' [make_sized_bounded_int_split_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sized_bounded_int_split_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
make_sized_bounded_int_split_sum <- function(
  size,
  bounds,
  .T = NULL
) {
  assert_features("contrib")

  # Standardize type arguments.
  .T <- parse_or_infer(type_name = .T, public_example = get_first(bounds))
  .T.bounds <- new_runtime_type(origin = "Tuple", args = list(.T, .T))

  log_ <- new_constructor_log("make_sized_bounded_int_split_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(size))
  rt_assert_is_similar(expected = .T.bounds, inferred = rt_infer(bounds))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sized_bounded_int_split_sum",
    size, bounds, .T, rt_parse(.T.bounds),
    log_, PACKAGE = "opendp")
  output
}

#' partial sized bounded int split sum constructor
#'
#' See documentation for [make_sized_bounded_int_split_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param size Number of records in input data.
#' @param bounds Tuple of lower and upper bounds for data in the input domain.
#' @param .T Atomic Input Type and Output Type
#' @return Transformation
#' @export
then_sized_bounded_int_split_sum <- function(
  lhs,
  size,
  bounds,
  .T = NULL
) {

  log_ <- new_constructor_log("then_sized_bounded_int_split_sum", "transformations", new_hashtab(
    list("size", "bounds", "T"),
    list(unbox2(size), lapply(bounds, unbox2), .T)
  ))

  make_chain_dyn(
    make_sized_bounded_int_split_sum(
      size = size,
      bounds = bounds,
      .T = .T),
    lhs,
    log_)
}


#' split dataframe constructor
#'
#' Make a Transformation that splits each record in a String into a `Vec<Vec<String>>`,
#' and loads the resulting table into a dataframe keyed by `col_names`.
#'
#'
#' Required features: `contrib`
#'
#' [make_split_dataframe in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_split_dataframe.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<String>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `DataFrameDomain<K>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#' @param separator The token(s) that separate entries in each record.
#' @param col_names Column names for each record entry.
#' @param .K categorical/hashable data type of column names
#' @return Transformation
#' @export
make_split_dataframe <- function(
  separator,
  col_names,
  .K = NULL
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .K <- parse_or_infer(type_name = .K, public_example = get_first(col_names))
  .T.col_names <- new_runtime_type(origin = "Vec", args = list(.K))

  log_ <- new_constructor_log("make_split_dataframe", "transformations", new_hashtab(
    list("separator", "col_names", "K"),
    list(separator, col_names, .K)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .T.col_names, inferred = rt_infer(col_names))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_split_dataframe",
    separator, col_names, .K, rt_parse(.T.col_names),
    log_, PACKAGE = "opendp")
  output
}

#' partial split dataframe constructor
#'
#' See documentation for [make_split_dataframe()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param separator The token(s) that separate entries in each record.
#' @param col_names Column names for each record entry.
#' @param .K categorical/hashable data type of column names
#' @return Transformation
#' @export
then_split_dataframe <- function(
  lhs,
  separator,
  col_names,
  .K = NULL
) {

  log_ <- new_constructor_log("then_split_dataframe", "transformations", new_hashtab(
    list("separator", "col_names", "K"),
    list(separator, col_names, .K)
  ))

  make_chain_dyn(
    make_split_dataframe(
      separator = separator,
      col_names = col_names,
      .K = .K),
    lhs,
    log_)
}


#' split lines constructor
#'
#' Make a Transformation that takes a string and splits it into a `Vec<String>` of its lines.
#'
#'
#' Required features: `contrib`
#'
#' [make_split_lines in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_split_lines.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `AtomDomain<String>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `VectorDomain<AtomDomain<String>>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#'
#' @return Transformation
#' @export
make_split_lines <- function(

) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_split_lines", "transformations", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_split_lines",
    log_, PACKAGE = "opendp")
  output
}

#' partial split lines constructor
#'
#' See documentation for [make_split_lines()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_split_lines <- function(
  lhs
) {

  log_ <- new_constructor_log("then_split_lines", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_split_lines(),
    lhs,
    log_)
}


#' split records constructor
#'
#' Make a Transformation that splits each record in a `Vec<String>` into a `Vec<Vec<String>>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_split_records in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_split_records.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<String>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `VectorDomain<VectorDomain<AtomDomain<String>>>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#' @param separator The token(s) that separate entries in each record.
#' @return Transformation
#' @export
make_split_records <- function(
  separator
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_split_records", "transformations", new_hashtab(
    list("separator"),
    list(separator)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_split_records",
    separator,
    log_, PACKAGE = "opendp")
  output
}

#' partial split records constructor
#'
#' See documentation for [make_split_records()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param separator The token(s) that separate entries in each record.
#' @return Transformation
#' @export
then_split_records <- function(
  lhs,
  separator
) {

  log_ <- new_constructor_log("then_split_records", "transformations", new_hashtab(
    list("separator"),
    list(separator)
  ))

  make_chain_dyn(
    make_split_records(
      separator = separator),
    lhs,
    log_)
}


#' subset by constructor
#'
#' Make a Transformation that subsets a dataframe by a boolean column.
#'
#'
#' Required features: `contrib`
#'
#' [make_subset_by in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_subset_by.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `DataFrameDomain<TK>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `DataFrameDomain<TK>`
#' * Output Metric:  `SymmetricDistance`
#'
#' @concept transformations
#' @param indicator_column name of the boolean column that indicates inclusion in the subset
#' @param keep_columns list of column names to apply subset to
#' @param .TK Type of the column name
#' @return Transformation
#' @export
make_subset_by <- function(
  indicator_column,
  keep_columns,
  .TK = NULL
) {
  .Deprecated(msg = "Use Polars instead.")
  assert_features("contrib")

  # Standardize type arguments.
  .TK <- parse_or_infer(type_name = .TK, public_example = indicator_column)
  .T.keep_columns <- new_runtime_type(origin = "Vec", args = list(.TK))

  log_ <- new_constructor_log("make_subset_by", "transformations", new_hashtab(
    list("indicator_column", "keep_columns", "TK"),
    list(indicator_column, keep_columns, .TK)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = .TK, inferred = rt_infer(indicator_column))
  rt_assert_is_similar(expected = .T.keep_columns, inferred = rt_infer(keep_columns))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_subset_by",
    indicator_column, keep_columns, .TK, rt_parse(.T.keep_columns),
    log_, PACKAGE = "opendp")
  output
}

#' partial subset by constructor
#'
#' See documentation for [make_subset_by()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param indicator_column name of the boolean column that indicates inclusion in the subset
#' @param keep_columns list of column names to apply subset to
#' @param .TK Type of the column name
#' @return Transformation
#' @export
then_subset_by <- function(
  lhs,
  indicator_column,
  keep_columns,
  .TK = NULL
) {

  log_ <- new_constructor_log("then_subset_by", "transformations", new_hashtab(
    list("indicator_column", "keep_columns", "TK"),
    list(indicator_column, keep_columns, .TK)
  ))

  make_chain_dyn(
    make_subset_by(
      indicator_column = indicator_column,
      keep_columns = keep_columns,
      .TK = .TK),
    lhs,
    log_)
}


#' sum constructor
#'
#' Make a Transformation that computes the sum of bounded data.
#' Use `make_clamp` to bound data.
#'
#' If dataset size is known, uses a restricted-sensitivity proof that takes advantage of known dataset size for better utility.
#'
#'
#' Required features: `contrib`
#'
#' [make_sum in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sum.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<T>>`
#' * Output Domain:  `MI`
#' * Input Metric:   `AtomDomain<T>`
#' * Output Metric:  `AbsoluteDistance<T>`
#'
#' @concept transformations
#' @param input_domain Domain of the input data.
#' @param input_metric One of `SymmetricDistance` or `InsertDeleteDistance`.
#' @return Transformation
#' @export
make_sum <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_sum", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sum",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial sum constructor
#'
#' See documentation for [make_sum()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_sum <- function(
  lhs
) {

  log_ <- new_constructor_log("then_sum", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_sum(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' sum of squared deviations constructor
#'
#' Make a Transformation that computes the sum of squared deviations of bounded data.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size.
#' Use `make_clamp` to bound data and `make_resize` to establish dataset size.
#'
#' | S (summation algorithm) | input type     |
#' | ----------------------- | -------------- |
#' | `Sequential<S::Item>`   | `Vec<S::Item>` |
#' | `Pairwise<S::Item>`     | `Vec<S::Item>` |
#'
#' `S::Item` is the type of all of the following:
#' each bound, each element in the input data, the output data, and the output sensitivity.
#'
#' For example, to construct a transformation that computes the SSD of `f32` half-precision floats,
#' set `S` to `Pairwise<f32>`.
#'
#'
#' Required features: `contrib`
#'
#' [make_sum_of_squared_deviations in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_sum_of_squared_deviations.html)
#'
#' **Citations:**
#'
#' * [CSVW22 Widespread Underestimation of Sensitivity...](https://arxiv.org/pdf/2207.10635.pdf)
#' * [DMNS06 Calibrating Noise to Sensitivity in Private Data Analysis](https://people.csail.mit.edu/asmith/PS/sensitivity-tcc-final.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param .S Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
#' @return Transformation
#' @export
make_sum_of_squared_deviations <- function(
  input_domain,
  input_metric,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom(get_type(input_domain))
  .S <- rt_substitute(.S, .T = .T)

  log_ <- new_constructor_log("make_sum_of_squared_deviations", "transformations", new_hashtab(
    list("input_domain", "input_metric", "S"),
    list(input_domain, input_metric, .S)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_sum_of_squared_deviations",
    input_domain, input_metric, .S, .T,
    log_, PACKAGE = "opendp")
  output
}

#' partial sum of squared deviations constructor
#'
#' See documentation for [make_sum_of_squared_deviations()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param .S Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
#' @return Transformation
#' @export
then_sum_of_squared_deviations <- function(
  lhs,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_sum_of_squared_deviations", "transformations", new_hashtab(
    list("S"),
    list(.S)
  ))

  make_chain_dyn(
    make_sum_of_squared_deviations(
      output_domain(lhs),
      output_metric(lhs),
      .S = .S),
    lhs,
    log_)
}


#' unordered constructor
#'
#' Make a Transformation that converts the ordered dataset metric `MI`
#' to the respective ordered dataset metric with a no-op.
#'
#' | `MI`                 | `MI::UnorderedMetric` |
#' | -------------------- | --------------------- |
#' | InsertDeleteDistance | SymmetricDistance     |
#' | HammingDistance      | ChangeOneDistance     |
#'
#'
#' Required features: `contrib`
#'
#' [make_unordered in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_unordered.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Domain:  `MI`
#' * Input Metric:   `D`
#' * Output Metric:  `MI::UnorderedMetric`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @return Transformation
#' @export
make_unordered <- function(
  input_domain,
  input_metric
) {
  assert_features("contrib")

  # No type arguments to standardize.
  log_ <- new_constructor_log("make_unordered", "transformations", new_hashtab(
    list("input_domain", "input_metric"),
    list(input_domain, input_metric)
  ))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_unordered",
    input_domain, input_metric,
    log_, PACKAGE = "opendp")
  output
}

#' partial unordered constructor
#'
#' See documentation for [make_unordered()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#'
#' @return Transformation
#' @export
then_unordered <- function(
  lhs
) {

  log_ <- new_constructor_log("then_unordered", "transformations", new_hashtab(
    list(),
    list()
  ))

  make_chain_dyn(
    make_unordered(
      output_domain(lhs),
      output_metric(lhs)),
    lhs,
    log_)
}


#' variance constructor
#'
#' Make a Transformation that computes the variance of bounded data.
#'
#' This uses a restricted-sensitivity proof that takes advantage of known dataset size.
#' Use `make_clamp` to bound data and `make_resize` to establish dataset size.
#'
#'
#' Required features: `contrib`
#'
#' [make_variance in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.3/opendp/transformations/fn.make_variance.html)
#'
#' **Citations:**
#'
#' * [DHK15 Differential Privacy for Social Science Inference](http://hona.kr/papers/files/DOrazioHonakerKingPrivacy.pdf)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<S::Item>>`
#' * Output Domain:  `SymmetricDistance`
#' * Input Metric:   `AtomDomain<S::Item>`
#' * Output Metric:  `AbsoluteDistance<S::Item>`
#'
#' @concept transformations
#' @param input_domain Domain of input data
#' @param input_metric Metric on input domain
#' @param ddof Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
#' @param .S Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
#' @return Transformation
#' @export
make_variance <- function(
  input_domain,
  input_metric,
  ddof = 1L,
  .S = "Pairwise<.T>"
) {
  assert_features("contrib")

  # Standardize type arguments.
  .S <- rt_parse(type_name = .S, generics = list(".T"))
  .T <- get_atom(get_type(input_domain))
  .S <- rt_substitute(.S, .T = .T)

  log_ <- new_constructor_log("make_variance", "transformations", new_hashtab(
    list("input_domain", "input_metric", "ddof", "S"),
    list(input_domain, input_metric, unbox2(ddof), .S)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = usize, inferred = rt_infer(ddof))

  # Call wrapper function.
  output <- .Call(
    "transformations__make_variance",
    input_domain, input_metric, ddof, .S, .T,
    log_, PACKAGE = "opendp")
  output
}

#' partial variance constructor
#'
#' See documentation for [make_variance()] for details.
#'
#' @concept transformations
#' @param lhs The prior transformation or metric space.
#' @param ddof Delta degrees of freedom. Set to 0 if not a sample, 1 for sample estimate.
#' @param .S Summation algorithm to use on data type `T`. One of `Sequential<T>` or `Pairwise<T>`.
#' @return Transformation
#' @export
then_variance <- function(
  lhs,
  ddof = 1L,
  .S = "Pairwise<.T>"
) {

  log_ <- new_constructor_log("then_variance", "transformations", new_hashtab(
    list("ddof", "S"),
    list(unbox2(ddof), .S)
  ))

  make_chain_dyn(
    make_variance(
      output_domain(lhs),
      output_metric(lhs),
      ddof = ddof,
      .S = .S),
    lhs,
    log_)
}
