# Auto-generated. Do not edit.

#' @include typing.R mod.R
NULL


#' Retrieve the inner privacy measure of an approximate privacy measure.
#'
#' @concept measures
#' @param privacy_measure The privacy measure to inspect
#' @return Measure
approximate_divergence_get_inner_measure <- function(
  privacy_measure
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("_approximate_divergence_get_inner_measure", "measures", new_hashtab(
    list("privacy_measure"),
    list(privacy_measure)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasure, inferred = rt_infer(privacy_measure))

  # Call wrapper function.
  output <- .Call(
    "measures___approximate_divergence_get_inner_measure",
    privacy_measure,
    log_, PACKAGE = "opendp")
  output
}


#' Check whether two measures are equal.
#'
#' @concept measures
#' @param left Measure to compare.
#' @param right Measure to compare.
#' @return bool
measure_equal <- function(
  left,
  right
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("_measure_equal", "measures", new_hashtab(
    list("left", "right"),
    list(left, right)
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = AnyMeasure, inferred = rt_infer(left))
  rt_assert_is_similar(expected = AnyMeasure, inferred = rt_infer(right))

  # Call wrapper function.
  output <- .Call(
    "measures___measure_equal",
    left, right,
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{\delta}-approximate PM-differential privacy.
#'
#' In the following definition, \eqn{d} corresponds to privacy parameters \eqn{(d', \delta)}
#' when also quantified over all adjacent datasets
#' (\eqn{d'} is the privacy parameter corresponding to privacy measure PM).
#' That is, \eqn{(d', \delta)} is no smaller than \eqn{d} (by product ordering),
#' over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' [approximate in Rust documentation.](https://docs.rs/opendp/0.14.1-dev.20260219.5/opendp/measures/struct.Approximate.html)
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and 2-tuple \eqn{d = (d', \delta)},
#' where \eqn{d'} is the distance with respect to privacy measure PM,
#' \eqn{Y, Y'} are \eqn{d}-close under the approximate PM measure whenever,
#' for any choice of \eqn{\delta \in [0, 1]},
#' there exist events \eqn{E} (depending on \eqn{Y}) and \eqn{E'} (depending on \eqn{Y'})
#' such that \eqn{\Pr[E] \ge 1 - \delta}, \eqn{\Pr[E'] \ge 1 - \delta}, and
#'
#' \eqn{D_{\mathrm{PM}}^\delta(Y|_E, Y'|_{E'}) = D_{\mathrm{PM}}(Y|_E, Y'|_{E'})}
#'
#' where \eqn{Y|_E} denotes the distribution of \eqn{Y} conditioned on the event \eqn{E}.
#'
#' Note that this \eqn{\delta} is not privacy parameter \eqn{\delta} until quantified over all adjacent datasets,
#' as is done in the definition of a measurement.
#'
#' @concept measures
#' @param measure inner privacy measure
#' @return ApproximateDivergence
#' @export
approximate <- function(
  measure
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("approximate", "measures", new_hashtab(
    list("measure"),
    list(measure)
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__approximate",
    measure,
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{(\epsilon, \delta)}-approximate differential privacy.
#'
#' In the following definition, \eqn{d} corresponds to \eqn{(\epsilon, \delta)} when also quantified over all adjacent datasets.
#' That is, \eqn{(\epsilon, \delta)} is no smaller than \eqn{d} (by product ordering),
#' over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and any 2-tuple \eqn{d} of non-negative numbers \eqn{\epsilon} and \eqn{\delta},
#' \eqn{Y, Y'} are \eqn{d}-close under the fixed smoothed max divergence measure whenever
#'
#' \eqn{D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon}.
#'
#' Note that this \eqn{\epsilon} and \eqn{\delta} are not privacy parameters \eqn{\epsilon} and \eqn{\delta} until quantified over all adjacent datasets,
#' as is done in the definition of a measurement.
#'
#' @concept measures
#'
#' @return Measure
#' @export
fixed_smoothed_max_divergence <- function(

) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("fixed_smoothed_max_divergence", "measures", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__fixed_smoothed_max_divergence",
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{\epsilon}-pure differential privacy.
#'
#' In the following proof definition, \eqn{d} corresponds to \eqn{\epsilon} when also quantified over all adjacent datasets.
#' That is, \eqn{\epsilon} is the greatest possible \eqn{d}
#' over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and any non-negative \eqn{d},
#' \eqn{Y, Y'} are \eqn{d}-close under the max divergence measure whenever
#'
#' \eqn{D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d}.
#'
#' @concept measures
#'
#' @return Measure
#' @export
max_divergence <- function(

) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("max_divergence", "measures", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__max_divergence",
    log_, PACKAGE = "opendp")
  output
}


#' Debug a `measure`.
#'
#' @concept measures
#' @param this The measure to debug (stringify).
#' @return str
#' @export
measure_debug <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("measure_debug", "measures", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__measure_debug",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Get the distance type of a `measure`.
#'
#' @concept measures
#' @param this The measure to retrieve the distance type from.
#' @return str
#' @export
measure_distance_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("measure_distance_type", "measures", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__measure_distance_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Get the type of a `measure`.
#'
#' @concept measures
#' @param this The measure to retrieve the type from.
#' @return str
#' @export
measure_type <- function(
  this
) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("measure_type", "measures", new_hashtab(
    list("this"),
    list(this)
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__measure_type",
    this,
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{\epsilon(\alpha)}-Rényi differential privacy.
#'
#' In the following proof definition, \eqn{d} corresponds to an RDP curve when also quantified over all adjacent datasets.
#' That is, an RDP curve \eqn{\epsilon(\alpha)} is no smaller than \eqn{d(\alpha)} for any possible choices of \eqn{\alpha},
#' and over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and any curve \eqn{d},
#' \eqn{Y, Y'} are \eqn{d}-close under the Rényi divergence measure if,
#' for any given \eqn{\alpha \in (1, \infty)},
#'
#' \eqn{D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha)}
#'
#' Note that this \eqn{\epsilon} and \eqn{\alpha} are not privacy parameters \eqn{\epsilon} and \eqn{\alpha} until quantified over all adjacent datasets,
#' as is done in the definition of a measurement.
#'
#' @concept measures
#'
#' @return Measure
#' @export
renyi_divergence <- function(

) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("renyi_divergence", "measures", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__renyi_divergence",
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{\epsilon(\delta)}-approximate differential privacy.
#'
#' In the following proof definition, \eqn{d} corresponds to a privacy profile when also quantified over all adjacent datasets.
#' That is, a privacy profile \eqn{\epsilon(\delta)} is no smaller than \eqn{d(\delta)} for all possible choices of \eqn{\delta},
#' and over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' The distance \eqn{d} is of type PrivacyProfile, so it can be invoked with an \eqn{\epsilon}
#' to retrieve the corresponding \eqn{\delta}.
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and any curve \eqn{d(\cdot)},
#' \eqn{Y, Y'} are \eqn{d}-close under the smoothed max divergence measure whenever,
#' for any choice of non-negative \eqn{\epsilon}, and \eqn{\delta = d(\epsilon)},
#'
#' \eqn{D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon}.
#'
#' Note that \eqn{\epsilon} and \eqn{\delta} are not privacy parameters \eqn{\epsilon} and \eqn{\delta} until quantified over all adjacent datasets,
#' as is done in the definition of a measurement.
#'
#' @concept measures
#'
#' @return Measure
#' @export
smoothed_max_divergence <- function(

) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("smoothed_max_divergence", "measures", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__smoothed_max_divergence",
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure with meaning defined by an OpenDP Library user (you).
#'
#' Any two instances of UserDivergence are equal if their string descriptors are equal.
#'
#'
#' Required features: `honest-but-curious`
#'
#' **Why honest-but-curious?:**
#'
#' The essential requirement of a privacy measure is that it is closed under postprocessing.
#' Your privacy measure `D` must satisfy that, for any pure function `f` and any two distributions `Y, Y'`, then \eqn{D(Y, Y') \ge D(f(Y), f(Y'))}.
#'
#' Beyond this, you should also consider whether your privacy measure can be used to provide meaningful privacy guarantees to your privacy units.
#'
#' @concept measures
#' @param descriptor A string description of the privacy measure.
#' @return Measure
#' @export
user_divergence <- function(
  descriptor
) {
  assert_features("honest-but-curious")

  # No type arguments to standardize.
  log_ <- new_constructor_log("user_divergence", "measures", new_hashtab(
    list("descriptor"),
    list(unbox2(descriptor))
  ))

  # Assert that arguments are correctly typed.
  rt_assert_is_similar(expected = String, inferred = rt_infer(descriptor))

  # Call wrapper function.
  output <- .Call(
    "measures__user_divergence",
    descriptor,
    log_, PACKAGE = "opendp")
  output
}


#' Privacy measure used to define \eqn{\rho}-zero concentrated differential privacy.
#'
#' In the following proof definition, \eqn{d} corresponds to \eqn{\rho} when also quantified over all adjacent datasets.
#' That is, \eqn{\rho} is the greatest possible \eqn{d}
#' over all pairs of adjacent datasets \eqn{x, x'} where \eqn{Y \sim M(x)}, \eqn{Y' \sim M(x')}.
#' \eqn{M(\cdot)} is a measurement (commonly known as a mechanism).
#' The measurement's input metric defines the notion of adjacency,
#' and the measurement's input domain defines the set of possible datasets.
#'
#' **Proof Definition:**
#'
#' For any two distributions \eqn{Y, Y'} and any non-negative \eqn{d},
#' \eqn{Y, Y'} are \eqn{d}-close under the zero-concentrated divergence measure if,
#' for every possible choice of \eqn{\alpha \in (1, \infty)},
#'
#' \eqn{D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha}.
#'
#' @concept measures
#'
#' @return Measure
#' @export
zero_concentrated_divergence <- function(

) {
  # No type arguments to standardize.
  log_ <- new_constructor_log("zero_concentrated_divergence", "measures", new_hashtab(
    list(),
    list()
  ))

  # Call wrapper function.
  output <- .Call(
    "measures__zero_concentrated_divergence",
    log_, PACKAGE = "opendp")
  output
}
