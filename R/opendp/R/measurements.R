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


#' base discrete exponential constructor
#'
#' Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
#'
#' [make_base_discrete_exponential in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_exponential.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `VectorDomain<AtomDomain<TIA>>`
#' * Output Type:    `usize`
#' * Input Metric:   `LInfDiffDistance<TIA>`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' **Proof Definition:**
#'
#' [(Proof Document)](https://docs.opendp.org/en/nightly/proofs/rust/src/measurements/discrete_exponential/make_base_discrete_exponential.pdf)
#'
#' @param input_domain Domain of the input vector. Must be a non-nullable VectorDomain.
#' @param input_metric Metric on the input domain. Must be LInfDiffDistance
#' @param temperature Higher temperatures are more private.
#' @param optimize Indicate whether to privately return the "Max" or "Min"
#' @param .QO Output Distance Type.
#' @return Measurement
#' @export
make_base_discrete_exponential <- function(
    input_domain,
    input_metric,
    temperature,
    optimize,
    .QO = NULL
) {
    assert_features("contrib", "floating-point")

    # Standardize type arguments.
    .QO <- parse_or_infer(type_name = .QO, public_example = temperature)

    log <- new_constructor_log("make_base_discrete_exponential", "measurements", new_hashtab(
        list("input_domain", "input_metric", "temperature", "optimize", "QO"),
        list(input_domain, input_metric, temperature, unbox2(optimize), .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(temperature))
    rt_assert_is_similar(expected = String, inferred = rt_infer(optimize))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_discrete_exponential",
        input_domain, input_metric, temperature, optimize, .QO,
        log, PACKAGE = "opendp")
    output
}

#' partial base discrete exponential constructor
#'
#' See documentation for [make_base_discrete_exponential()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param temperature Higher temperatures are more private.
#' @param optimize Indicate whether to privately return the "Max" or "Min"
#' @param .QO Output Distance Type.
#' @return Measurement
#' @export
then_base_discrete_exponential <- function(
    lhs,
    temperature,
    optimize,
    .QO = NULL
) {

    log <- new_constructor_log("then_base_discrete_exponential", "measurements", new_hashtab(
        list("temperature", "optimize", "QO"),
        list(temperature, unbox2(optimize), .QO)
    ))

    make_chain_dyn(
        make_base_discrete_exponential(
            output_domain(lhs),
            output_metric(lhs),
            temperature = temperature,
            optimize = optimize,
            .QO = .QO),
        lhs,
        log)
}


#' base discrete gaussian constructor
#'
#' Make a Measurement that adds noise from the discrete_gaussian(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)`                | `T`          | `absolute_distance(QI)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(QI)`       |
#'
#' [make_base_discrete_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_gaussian.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MO`
#'
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param .MO Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
#' @return Measurement
#' @export
make_base_discrete_gaussian <- function(
    input_domain,
    input_metric,
    scale,
    .MO = "ZeroConcentratedDivergence<.QO>"
) {
    assert_features("contrib")

    # Standardize type arguments.
    .MO <- rt_parse(type_name = .MO, generics = list(".QO"))
    .QO <- get_atom_or_infer(.MO, scale)
    .MO <- rt_substitute(.MO, .QO = .QO)

    log <- new_constructor_log("make_base_discrete_gaussian", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "MO"),
        list(input_domain, input_metric, scale, .MO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_discrete_gaussian",
        input_domain, input_metric, scale, .MO, .QO,
        log, PACKAGE = "opendp")
    output
}

#' partial base discrete gaussian constructor
#'
#' See documentation for [make_base_discrete_gaussian()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param .MO Output measure. The only valid measure is `ZeroConcentratedDivergence<QO>`, but QO can be any float.
#' @return Measurement
#' @export
then_base_discrete_gaussian <- function(
    lhs,
    scale,
    .MO = "ZeroConcentratedDivergence<.QO>"
) {

    log <- new_constructor_log("then_base_discrete_gaussian", "measurements", new_hashtab(
        list("scale", "MO"),
        list(scale, .MO)
    ))

    make_chain_dyn(
        make_base_discrete_gaussian(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            .MO = .MO),
        lhs,
        log)
}


#' base discrete laplace constructor
#'
#' Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' This uses `make_base_discrete_laplace_cks20` if scale is greater than 10, otherwise it uses `make_base_discrete_laplace_linear`.
#'
#' [make_base_discrete_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace.html)
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
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
make_base_discrete_laplace <- function(
    input_domain,
    input_metric,
    scale,
    .QO = NULL
) {
    assert_features("contrib")

    # Standardize type arguments.
    .QO <- parse_or_infer(type_name = .QO, public_example = scale)

    log <- new_constructor_log("make_base_discrete_laplace", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "QO"),
        list(input_domain, input_metric, scale, .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_discrete_laplace",
        input_domain, input_metric, scale, .QO,
        log, PACKAGE = "opendp")
    output
}

#' partial base discrete laplace constructor
#'
#' See documentation for [make_base_discrete_laplace()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
then_base_discrete_laplace <- function(
    lhs,
    scale,
    .QO = NULL
) {

    log <- new_constructor_log("then_base_discrete_laplace", "measurements", new_hashtab(
        list("scale", "QO"),
        list(scale, .QO)
    ))

    make_chain_dyn(
        make_base_discrete_laplace(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            .QO = .QO),
        lhs,
        log)
}


#' base discrete laplace cks20 constructor
#'
#' Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
#' using an efficient algorithm on rational bignums.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' [make_base_discrete_laplace_cks20 in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_cks20.html)
#'
#' **Citations:**
#'
#' * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' @param input_domain undocumented
#' @param input_metric undocumented
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale.
#' @return Measurement
#' @export
make_base_discrete_laplace_cks20 <- function(
    input_domain,
    input_metric,
    scale,
    .QO = NULL
) {
    assert_features("contrib")

    # Standardize type arguments.
    .QO <- parse_or_infer(type_name = .QO, public_example = scale)

    log <- new_constructor_log("make_base_discrete_laplace_cks20", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "QO"),
        list(input_domain, input_metric, scale, .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_discrete_laplace_cks20",
        input_domain, input_metric, scale, .QO,
        log, PACKAGE = "opendp")
    output
}

#' partial base discrete laplace cks20 constructor
#'
#' See documentation for [make_base_discrete_laplace_cks20()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale.
#' @return Measurement
#' @export
then_base_discrete_laplace_cks20 <- function(
    lhs,
    scale,
    .QO = NULL
) {

    log <- new_constructor_log("then_base_discrete_laplace_cks20", "measurements", new_hashtab(
        list("scale", "QO"),
        list(scale, .QO)
    ))

    make_chain_dyn(
        make_base_discrete_laplace_cks20(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            .QO = .QO),
        lhs,
        log)
}


#' base discrete laplace linear constructor
#'
#' Make a Measurement that adds noise from the discrete_laplace(`scale`) distribution to the input,
#' using a linear-time algorithm on finite data types.
#'
#' This algorithm can be executed in constant time if bounds are passed.
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' [make_base_discrete_laplace_linear in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_discrete_laplace_linear.html)
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
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
#' @param bounds Set bounds on the count to make the algorithm run in constant-time.
#' @param .QO Data type of the scale and output distance.
#' @return Measurement
#' @export
make_base_discrete_laplace_linear <- function(
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

    log <- new_constructor_log("make_base_discrete_laplace_linear", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "bounds", "QO"),
        list(input_domain, input_metric, scale, bounds, .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))
    rt_assert_is_similar(expected = .OptionT, inferred = rt_infer(bounds))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_discrete_laplace_linear",
        input_domain, input_metric, scale, bounds, .QO, .T, .OptionT,
        log, PACKAGE = "opendp")
    output
}

#' partial base discrete laplace linear constructor
#'
#' See documentation for [make_base_discrete_laplace_linear()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the distribution. `scale` == standard_deviation / sqrt(2).
#' @param bounds Set bounds on the count to make the algorithm run in constant-time.
#' @param .QO Data type of the scale and output distance.
#' @return Measurement
#' @export
then_base_discrete_laplace_linear <- function(
    lhs,
    scale,
    bounds = NULL,
    .QO = NULL
) {

    log <- new_constructor_log("then_base_discrete_laplace_linear", "measurements", new_hashtab(
        list("scale", "bounds", "QO"),
        list(scale, bounds, .QO)
    ))

    make_chain_dyn(
        make_base_discrete_laplace_linear(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            bounds = bounds,
            .QO = .QO),
        lhs,
        log)
}


#' base gaussian constructor
#'
#' Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l2_distance(T)`       |
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#' [make_base_gaussian in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_gaussian.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MO`
#'
#' @param input_domain Domain of the data type to be privatized. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`.
#' @param input_metric Metric of the data type to be privatized. Valid values are `AbsoluteDistance<T>` or `L2Distance<T>`.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @export
make_base_gaussian <- function(
    input_domain,
    input_metric,
    scale,
    k = -1074L,
    .MO = "ZeroConcentratedDivergence<.T>"
) {
    assert_features("contrib")

    # Standardize type arguments.
    .MO <- rt_parse(type_name = .MO, generics = list(".T"))
    .T <- get_atom_or_infer(get_carrier_type(input_domain), scale)
    .MO <- rt_substitute(.MO, .T = .T)

    log <- new_constructor_log("make_base_gaussian", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "k", "MO"),
        list(input_domain, input_metric, scale, unbox2(k), .MO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
    rt_assert_is_similar(expected = i32, inferred = rt_infer(k))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_gaussian",
        input_domain, input_metric, scale, k, .MO, .T,
        log, PACKAGE = "opendp")
    output
}

#' partial base gaussian constructor
#'
#' See documentation for [make_base_gaussian()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param k The noise granularity in terms of 2^k.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @export
then_base_gaussian <- function(
    lhs,
    scale,
    k = -1074L,
    .MO = "ZeroConcentratedDivergence<.T>"
) {

    log <- new_constructor_log("then_base_gaussian", "measurements", new_hashtab(
        list("scale", "k", "MO"),
        list(scale, unbox2(k), .MO)
    ))

    make_chain_dyn(
        make_base_gaussian(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            k = k,
            .MO = .MO),
        lhs,
        log)
}


#' base geometric constructor
#'
#' An alias for `make_base_discrete_laplace_linear`.
#' If you don't need timing side-channel protections via `bounds`,
#' `make_base_discrete_laplace` is more efficient.
#'
#' [make_base_geometric in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_geometric.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MaxDivergence<QO>`
#'
#' @param input_domain undocumented
#' @param input_metric undocumented
#' @param scale undocumented
#' @param bounds undocumented
#' @param .QO undocumented
#' @return Measurement
#' @export
make_base_geometric <- function(
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

    log <- new_constructor_log("make_base_geometric", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "bounds", "QO"),
        list(input_domain, input_metric, scale, bounds, .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .QO, inferred = rt_infer(scale))
    rt_assert_is_similar(expected = .OptionT, inferred = rt_infer(bounds))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_geometric",
        input_domain, input_metric, scale, bounds, .QO, .T, .OptionT,
        log, PACKAGE = "opendp")
    output
}

#' partial base geometric constructor
#'
#' See documentation for [make_base_geometric()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale undocumented
#' @param bounds undocumented
#' @param .QO undocumented
#' @return Measurement
#' @export
then_base_geometric <- function(
    lhs,
    scale,
    bounds = NULL,
    .QO = NULL
) {

    log <- new_constructor_log("then_base_geometric", "measurements", new_hashtab(
        list("scale", "bounds", "QO"),
        list(scale, bounds, .QO)
    ))

    make_chain_dyn(
        make_base_geometric(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            bounds = bounds,
            .QO = .QO),
        lhs,
        log)
}


#' base laplace constructor
#'
#' Make a Measurement that adds noise from the Laplace(`scale`) distribution to a scalar value.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#' [make_base_laplace in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_laplace.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `D`
#' * Output Type:    `D::Carrier`
#' * Input Metric:   `D::InputMetric`
#' * Output Measure: `MaxDivergence<D::Atom>`
#'
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
make_base_laplace <- function(
    input_domain,
    input_metric,
    scale,
    k = -1074L
) {
    assert_features("contrib")

    # Standardize type arguments.
    .T <- get_atom_or_infer(get_carrier_type(input_domain), scale)

    log <- new_constructor_log("make_base_laplace", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "k"),
        list(input_domain, input_metric, scale, unbox2(k))
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .T, inferred = rt_infer(scale))
    rt_assert_is_similar(expected = i32, inferred = rt_infer(k))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_laplace",
        input_domain, input_metric, scale, k, .T,
        log, PACKAGE = "opendp")
    output
}

#' partial base laplace constructor
#'
#' See documentation for [make_base_laplace()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
then_base_laplace <- function(
    lhs,
    scale,
    k = -1074L
) {

    log <- new_constructor_log("then_base_laplace", "measurements", new_hashtab(
        list("scale", "k"),
        list(scale, unbox2(k))
    ))

    make_chain_dyn(
        make_base_laplace(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            k = k),
        lhs,
        log)
}


#' base laplace threshold constructor
#'
#' Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
#'
#' This function takes a noise granularity in terms of 2^k.
#' Larger granularities are more computationally efficient, but have a looser privacy map.
#' If k is not set, k defaults to the smallest granularity.
#'
#' [make_base_laplace_threshold in Rust documentation.](https://docs.rs/opendp/latest/opendp/measurements/fn.make_base_laplace_threshold.html)
#'
#' **Supporting Elements:**
#'
#' * Input Domain:   `MapDomain<AtomDomain<TK>, AtomDomain<TV>>`
#' * Output Type:    `HashMap<TK, TV>`
#' * Input Metric:   `L1Distance<TV>`
#' * Output Measure: `FixedSmoothedMaxDivergence<TV>`
#'
#' @param input_domain Domain of the input.
#' @param input_metric Metric for the input domain.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
make_base_laplace_threshold <- function(
    input_domain,
    input_metric,
    scale,
    threshold,
    k = -1074L
) {
    assert_features("contrib", "floating-point")

    # Standardize type arguments.
    .TV <- get_distance_type(input_metric)

    log <- new_constructor_log("make_base_laplace_threshold", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "threshold", "k"),
        list(input_domain, input_metric, scale, threshold, unbox2(k))
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .TV, inferred = rt_infer(scale))
    rt_assert_is_similar(expected = .TV, inferred = rt_infer(threshold))
    rt_assert_is_similar(expected = i32, inferred = rt_infer(k))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_base_laplace_threshold",
        input_domain, input_metric, scale, threshold, k, .TV,
        log, PACKAGE = "opendp")
    output
}

#' partial base laplace threshold constructor
#'
#' See documentation for [make_base_laplace_threshold()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param threshold Exclude counts that are less than this minimum value.
#' @param k The noise granularity in terms of 2^k.
#' @return Measurement
#' @export
then_base_laplace_threshold <- function(
    lhs,
    scale,
    threshold,
    k = -1074L
) {

    log <- new_constructor_log("then_base_laplace_threshold", "measurements", new_hashtab(
        list("scale", "threshold", "k"),
        list(scale, threshold, unbox2(k))
    ))

    make_chain_dyn(
        make_base_laplace_threshold(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            threshold = threshold,
            k = k),
        lhs,
        log)
}


#' gaussian constructor
#'
#' Make a Measurement that adds noise from the gaussian(`scale`) distribution to the input.
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
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @export
make_gaussian <- function(
    input_domain,
    input_metric,
    scale,
    .MO = "ZeroConcentratedDivergence<.QO>"
) {
    assert_features("contrib")

    # Standardize type arguments.
    .MO <- rt_parse(type_name = .MO, generics = list(".QO"))
    .QO <- get_atom_or_infer(.MO, scale)
    .MO <- rt_substitute(.MO, .QO = .QO)
    .T.scale <- get_atom(.MO)

    log <- new_constructor_log("make_gaussian", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "MO"),
        list(input_domain, input_metric, scale, .MO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .T.scale, inferred = rt_infer(scale))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_gaussian",
        input_domain, input_metric, scale, .MO, .QO, rt_parse(.T.scale),
        log, PACKAGE = "opendp")
    output
}

#' partial gaussian constructor
#'
#' See documentation for [make_gaussian()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the gaussian distribution. `scale` == standard_deviation.
#' @param .MO Output Measure. The only valid measure is `ZeroConcentratedDivergence<T>`.
#' @return Measurement
#' @export
then_gaussian <- function(
    lhs,
    scale,
    .MO = "ZeroConcentratedDivergence<.QO>"
) {

    log <- new_constructor_log("then_gaussian", "measurements", new_hashtab(
        list("scale", "MO"),
        list(scale, .MO)
    ))

    make_chain_dyn(
        make_gaussian(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            .MO = .MO),
        lhs,
        log)
}


#' laplace constructor
#'
#' Make a Measurement that adds noise from the laplace(`scale`) distribution to the input.
#'
#' Valid inputs for `input_domain` and `input_metric` are:
#'
#' | `input_domain`                  | input type   | `input_metric`         |
#' | ------------------------------- | ------------ | ---------------------- |
#' | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
#' | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
#'
#' This uses `make_base_laplace` if `T` is float, otherwise it uses `make_base_discrete_laplace`.
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
#' @param input_domain Domain of the data type to be privatized.
#' @param input_metric Metric of the data type to be privatized.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
make_laplace <- function(
    input_domain,
    input_metric,
    scale,
    .QO = "float"
) {
    assert_features("contrib")

    # Standardize type arguments.
    .QO <- rt_parse(type_name = .QO)
    .T.scale <- get_atom(.QO)

    log <- new_constructor_log("make_laplace", "measurements", new_hashtab(
        list("input_domain", "input_metric", "scale", "QO"),
        list(input_domain, input_metric, scale, .QO)
    ))

    # Assert that arguments are correctly typed.
    rt_assert_is_similar(expected = .T.scale, inferred = rt_infer(scale))

    # Call wrapper function.
    output <- .Call(
        "measurements__make_laplace",
        input_domain, input_metric, scale, .QO, rt_parse(.T.scale),
        log, PACKAGE = "opendp")
    output
}

#' partial laplace constructor
#'
#' See documentation for [make_laplace()] for details.
#'
#' @param lhs The prior transformation or metric space.
#' @param scale Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
#' @param .QO Data type of the output distance and scale. `f32` or `f64`.
#' @return Measurement
#' @export
then_laplace <- function(
    lhs,
    scale,
    .QO = "float"
) {

    log <- new_constructor_log("then_laplace", "measurements", new_hashtab(
        list("scale", "QO"),
        list(scale, .QO)
    ))

    make_chain_dyn(
        make_laplace(
            output_domain(lhs),
            output_metric(lhs),
            scale = scale,
            .QO = .QO),
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
