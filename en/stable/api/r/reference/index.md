# Package index

## Accuracy

The `accuracy` module provides functions for converting between accuracy
and scale parameters.

- [`accuracy_to_discrete_gaussian_scale()`](https://docs.opendp.org/reference/accuracy_to_discrete_gaussian_scale.md)
  :

  Convert a desired `accuracy` (tolerance) into a discrete gaussian
  noise scale at a statistical significance level `alpha`.

- [`accuracy_to_discrete_laplacian_scale()`](https://docs.opendp.org/reference/accuracy_to_discrete_laplacian_scale.md)
  :

  Convert a desired `accuracy` (tolerance) into a discrete Laplacian
  noise scale at a statistical significance level `alpha`.

- [`accuracy_to_gaussian_scale()`](https://docs.opendp.org/reference/accuracy_to_gaussian_scale.md)
  :

  Convert a desired `accuracy` (tolerance) into a gaussian noise scale
  at a statistical significance level `alpha`.

- [`accuracy_to_laplacian_scale()`](https://docs.opendp.org/reference/accuracy_to_laplacian_scale.md)
  :

  Convert a desired `accuracy` (tolerance) into a Laplacian noise scale
  at a statistical significance level `alpha`.

- [`discrete_gaussian_scale_to_accuracy()`](https://docs.opendp.org/reference/discrete_gaussian_scale_to_accuracy.md)
  :

  Convert a discrete gaussian scale into an accuracy estimate
  (tolerance) at a statistical significance level `alpha`.

- [`discrete_laplacian_scale_to_accuracy()`](https://docs.opendp.org/reference/discrete_laplacian_scale_to_accuracy.md)
  :

  Convert a discrete Laplacian scale into an accuracy estimate
  (tolerance) at a statistical significance level `alpha`.

- [`gaussian_scale_to_accuracy()`](https://docs.opendp.org/reference/gaussian_scale_to_accuracy.md)
  :

  Convert a gaussian scale into an accuracy estimate (tolerance) at a
  statistical significance level `alpha`.

- [`laplacian_scale_to_accuracy()`](https://docs.opendp.org/reference/laplacian_scale_to_accuracy.md)
  :

  Convert a Laplacian scale into an accuracy estimate (tolerance) at a
  statistical significance level `alpha`.

## Combinators

The `combinators` module provides functions for combining
transformations and measurements.

- [`make_adaptive_composition()`](https://docs.opendp.org/reference/make_adaptive_composition.md)
  : adaptive composition constructor
- [`make_approximate()`](https://docs.opendp.org/reference/make_approximate.md)
  : approximate constructor
- [`make_basic_composition()`](https://docs.opendp.org/reference/make_basic_composition.md)
  : basic composition constructor
- [`make_chain_mt()`](https://docs.opendp.org/reference/make_chain_mt.md)
  : chain mt constructor
- [`make_chain_pm()`](https://docs.opendp.org/reference/make_chain_pm.md)
  : chain pm constructor
- [`make_chain_tt()`](https://docs.opendp.org/reference/make_chain_tt.md)
  : chain tt constructor
- [`make_composition()`](https://docs.opendp.org/reference/make_composition.md)
  : composition constructor
- [`make_fix_delta()`](https://docs.opendp.org/reference/make_fix_delta.md)
  : fix delta constructor
- [`make_fixed_approxDP_to_approxDP()`](https://docs.opendp.org/reference/make_fixed_approxDP_to_approxDP.md)
  : fixed approxDP to approxDP constructor
- [`make_fully_adaptive_composition()`](https://docs.opendp.org/reference/make_fully_adaptive_composition.md)
  : fully adaptive composition constructor
- [`make_population_amplification()`](https://docs.opendp.org/reference/make_population_amplification.md)
  : population amplification constructor
- [`make_privacy_filter()`](https://docs.opendp.org/reference/make_privacy_filter.md)
  : privacy filter constructor
- [`make_pureDP_to_zCDP()`](https://docs.opendp.org/reference/make_pureDP_to_zCDP.md)
  : pureDP to zCDP constructor
- [`make_select_private_candidate()`](https://docs.opendp.org/reference/make_select_private_candidate.md)
  : select private candidate constructor
- [`make_sequential_composition()`](https://docs.opendp.org/reference/make_sequential_composition.md)
  : sequential composition constructor
- [`make_zCDP_to_approxDP()`](https://docs.opendp.org/reference/make_zCDP_to_approxDP.md)
  : zCDP to approxDP constructor
- [`then_adaptive_composition()`](https://docs.opendp.org/reference/then_adaptive_composition.md)
  : partial adaptive composition constructor
- [`then_approximate()`](https://docs.opendp.org/reference/then_approximate.md)
  : partial approximate constructor
- [`then_basic_composition()`](https://docs.opendp.org/reference/then_basic_composition.md)
  : partial basic composition constructor
- [`then_chain_mt()`](https://docs.opendp.org/reference/then_chain_mt.md)
  : partial chain mt constructor
- [`then_chain_pm()`](https://docs.opendp.org/reference/then_chain_pm.md)
  : partial chain pm constructor
- [`then_chain_tt()`](https://docs.opendp.org/reference/then_chain_tt.md)
  : partial chain tt constructor
- [`then_composition()`](https://docs.opendp.org/reference/then_composition.md)
  : partial composition constructor
- [`then_fix_delta()`](https://docs.opendp.org/reference/then_fix_delta.md)
  : partial fix delta constructor
- [`then_fixed_approxDP_to_approxDP()`](https://docs.opendp.org/reference/then_fixed_approxDP_to_approxDP.md)
  : partial fixed approxDP to approxDP constructor
- [`then_fully_adaptive_composition()`](https://docs.opendp.org/reference/then_fully_adaptive_composition.md)
  : partial fully adaptive composition constructor
- [`then_population_amplification()`](https://docs.opendp.org/reference/then_population_amplification.md)
  : partial population amplification constructor
- [`then_privacy_filter()`](https://docs.opendp.org/reference/then_privacy_filter.md)
  : partial privacy filter constructor
- [`then_pureDP_to_zCDP()`](https://docs.opendp.org/reference/then_pureDP_to_zCDP.md)
  : partial pureDP to zCDP constructor
- [`then_select_private_candidate()`](https://docs.opendp.org/reference/then_select_private_candidate.md)
  : partial select private candidate constructor
- [`then_sequential_composition()`](https://docs.opendp.org/reference/then_sequential_composition.md)
  : partial sequential composition constructor
- [`then_zCDP_to_approxDP()`](https://docs.opendp.org/reference/then_zCDP_to_approxDP.md)
  : partial zCDP to approxDP constructor

## Core

The `core` module provides functions for accessing the fields of
transformations and measurements.

- [`function_eval()`](https://docs.opendp.org/reference/function_eval.md)
  :

  Eval the `function` with `arg`.

- [`measurement_check()`](https://docs.opendp.org/reference/measurement_check.md)
  :

  Check the privacy relation of the `measurement` at the given `d_in`,
  `d_out`

- [`measurement_function()`](https://docs.opendp.org/reference/measurement_function.md)
  : Get the function from a measurement.

- [`measurement_input_carrier_type()`](https://docs.opendp.org/reference/measurement_input_carrier_type.md)
  :

  Get the input (carrier) data type of `this`.

- [`measurement_input_distance_type()`](https://docs.opendp.org/reference/measurement_input_distance_type.md)
  :

  Get the input distance type of `measurement`.

- [`measurement_input_domain()`](https://docs.opendp.org/reference/measurement_input_domain.md)
  :

  Get the input domain from a `measurement`.

- [`measurement_input_metric()`](https://docs.opendp.org/reference/measurement_input_metric.md)
  :

  Get the input domain from a `measurement`.

- [`measurement_invoke()`](https://docs.opendp.org/reference/measurement_invoke.md)
  :

  Invoke the `measurement` with `arg`. Returns a differentially private
  release.

- [`measurement_map()`](https://docs.opendp.org/reference/measurement_map.md)
  :

  Use the `measurement` to map a given `d_in` to `d_out`.

- [`measurement_output_distance_type()`](https://docs.opendp.org/reference/measurement_output_distance_type.md)
  :

  Get the output distance type of `measurement`.

- [`measurement_output_measure()`](https://docs.opendp.org/reference/measurement_output_measure.md)
  :

  Get the output domain from a `measurement`.

- [`odometer_input_carrier_type()`](https://docs.opendp.org/reference/odometer_input_carrier_type.md)
  :

  Get the input (carrier) data type of `this`.

- [`odometer_input_domain()`](https://docs.opendp.org/reference/odometer_input_domain.md)
  :

  Get the input domain from a `odometer`.

- [`odometer_input_metric()`](https://docs.opendp.org/reference/odometer_input_metric.md)
  :

  Get the input domain from a `odometer`.

- [`odometer_invoke()`](https://docs.opendp.org/reference/odometer_invoke.md)
  :

  Invoke the `odometer` with `arg`. Returns a differentially private
  release.

- [`odometer_output_measure()`](https://docs.opendp.org/reference/odometer_output_measure.md)
  :

  Get the output domain from a `odometer`.

- [`odometer_queryable_invoke()`](https://docs.opendp.org/reference/odometer_queryable_invoke.md)
  :

  Eval the odometer `queryable` with an invoke `query`.

- [`odometer_queryable_invoke_type()`](https://docs.opendp.org/reference/odometer_queryable_invoke_type.md)
  :

  Get the invoke query type of an odometer `queryable`.

- [`odometer_queryable_privacy_loss()`](https://docs.opendp.org/reference/odometer_queryable_privacy_loss.md)
  :

  Retrieve the privacy loss of an odometer `queryable`.

- [`odometer_queryable_privacy_loss_type()`](https://docs.opendp.org/reference/odometer_queryable_privacy_loss_type.md)
  :

  Get the map query type of an odometer `queryable`.

- [`queryable_eval()`](https://docs.opendp.org/reference/queryable_eval.md)
  :

  Eval the `queryable` with `query`. Returns a differentially private
  release.

- [`queryable_query_type()`](https://docs.opendp.org/reference/queryable_query_type.md)
  :

  Get the query type of `queryable`.

- [`transformation_check()`](https://docs.opendp.org/reference/transformation_check.md)
  :

  Check the privacy relation of the `measurement` at the given `d_in`,
  `d_out`

- [`transformation_function()`](https://docs.opendp.org/reference/transformation_function.md)
  : Get the function from a transformation.

- [`transformation_input_carrier_type()`](https://docs.opendp.org/reference/transformation_input_carrier_type.md)
  :

  Get the input (carrier) data type of `this`.

- [`transformation_input_distance_type()`](https://docs.opendp.org/reference/transformation_input_distance_type.md)
  :

  Get the input distance type of `transformation`.

- [`transformation_input_domain()`](https://docs.opendp.org/reference/transformation_input_domain.md)
  :

  Get the input domain from a `transformation`.

- [`transformation_input_metric()`](https://docs.opendp.org/reference/transformation_input_metric.md)
  :

  Get the input domain from a `transformation`.

- [`transformation_invoke()`](https://docs.opendp.org/reference/transformation_invoke.md)
  :

  Invoke the `transformation` with `arg`. Returns a differentially
  private release.

- [`transformation_map()`](https://docs.opendp.org/reference/transformation_map.md)
  :

  Use the `transformation` to map a given `d_in` to `d_out`.

- [`transformation_output_distance_type()`](https://docs.opendp.org/reference/transformation_output_distance_type.md)
  :

  Get the output distance type of `transformation`.

- [`transformation_output_domain()`](https://docs.opendp.org/reference/transformation_output_domain.md)
  :

  Get the output domain from a `transformation`.

- [`transformation_output_metric()`](https://docs.opendp.org/reference/transformation_output_metric.md)
  :

  Get the output domain from a `transformation`.

## Domains

The `domains` module provides functions for creating and using domains.

- [`atom_domain()`](https://docs.opendp.org/reference/atom_domain.md) :

  Construct an instance of `AtomDomain`.

- [`atom_domain_get_bounds_closed()`](https://docs.opendp.org/reference/atom_domain_get_bounds_closed.md)
  :

  Retrieve bounds from an AtomDomain

- [`atom_domain_nan()`](https://docs.opendp.org/reference/atom_domain_nan.md)
  :

  Retrieve whether members of AtomDomain may be NaN.

- [`bitvector_domain()`](https://docs.opendp.org/reference/bitvector_domain.md)
  :

  Construct an instance of `BitVectorDomain`.

- [`domain_carrier_type()`](https://docs.opendp.org/reference/domain_carrier_type.md)
  :

  Get the carrier type of a `domain`.

- [`domain_debug()`](https://docs.opendp.org/reference/domain_debug.md)
  :

  Debug a `domain`.

- [`domain_equal()`](https://docs.opendp.org/reference/domain_equal.md)
  : Check whether two domains are equal.

- [`domain_type()`](https://docs.opendp.org/reference/domain_type.md) :

  Get the type of a `domain`.

- [`map_domain()`](https://docs.opendp.org/reference/map_domain.md) :

  Construct an instance of `MapDomain`.

- [`member()`](https://docs.opendp.org/reference/member.md) :

  Check membership in a `domain`.

- [`option_domain()`](https://docs.opendp.org/reference/option_domain.md)
  :

  Construct an instance of `OptionDomain`.

- [`option_domain_get_element_domain()`](https://docs.opendp.org/reference/option_domain_get_element_domain.md)
  : Retrieve the element domain of the option domain.

- [`vector_domain()`](https://docs.opendp.org/reference/vector_domain.md)
  :

  Construct an instance of `VectorDomain`.

- [`vector_domain_get_element_domain()`](https://docs.opendp.org/reference/vector_domain_get_element_domain.md)
  : Retrieve the element domain of the vector domain.

- [`vector_domain_get_size()`](https://docs.opendp.org/reference/vector_domain_get_size.md)
  : Retrieve the size of vectors in the vector domain.

## Measurements

The `measurements` module provides functions that apply calibrated noise
to data to ensure differential privacy.

- [`make_alp_queryable()`](https://docs.opendp.org/reference/make_alp_queryable.md)
  : alp queryable constructor
- [`make_canonical_noise()`](https://docs.opendp.org/reference/make_canonical_noise.md)
  : canonical noise constructor
- [`make_gaussian()`](https://docs.opendp.org/reference/make_gaussian.md)
  : gaussian constructor
- [`make_gaussian_threshold()`](https://docs.opendp.org/reference/make_gaussian_threshold.md)
  : gaussian threshold constructor
- [`make_geometric()`](https://docs.opendp.org/reference/make_geometric.md)
  : geometric constructor
- [`make_laplace()`](https://docs.opendp.org/reference/make_laplace.md)
  : laplace constructor
- [`make_laplace_threshold()`](https://docs.opendp.org/reference/make_laplace_threshold.md)
  : laplace threshold constructor
- [`make_noise()`](https://docs.opendp.org/reference/make_noise.md) :
  noise constructor
- [`make_noise_threshold()`](https://docs.opendp.org/reference/make_noise_threshold.md)
  : noise threshold constructor
- [`make_noisy_max()`](https://docs.opendp.org/reference/make_noisy_max.md)
  : noisy max constructor
- [`make_noisy_top_k()`](https://docs.opendp.org/reference/make_noisy_top_k.md)
  : noisy top k constructor
- [`make_private_quantile()`](https://docs.opendp.org/reference/make_private_quantile.md)
  : private quantile constructor
- [`make_randomized_response()`](https://docs.opendp.org/reference/make_randomized_response.md)
  : randomized response constructor
- [`make_randomized_response_bitvec()`](https://docs.opendp.org/reference/make_randomized_response_bitvec.md)
  : randomized response bitvec constructor
- [`make_randomized_response_bool()`](https://docs.opendp.org/reference/make_randomized_response_bool.md)
  : randomized response bool constructor
- [`make_report_noisy_max_gumbel()`](https://docs.opendp.org/reference/make_report_noisy_max_gumbel.md)
  : report noisy max gumbel constructor
- [`then_alp_queryable()`](https://docs.opendp.org/reference/then_alp_queryable.md)
  : partial alp queryable constructor
- [`then_canonical_noise()`](https://docs.opendp.org/reference/then_canonical_noise.md)
  : partial canonical noise constructor
- [`then_gaussian()`](https://docs.opendp.org/reference/then_gaussian.md)
  : partial gaussian constructor
- [`then_gaussian_threshold()`](https://docs.opendp.org/reference/then_gaussian_threshold.md)
  : partial gaussian threshold constructor
- [`then_geometric()`](https://docs.opendp.org/reference/then_geometric.md)
  : partial geometric constructor
- [`then_laplace()`](https://docs.opendp.org/reference/then_laplace.md)
  : partial laplace constructor
- [`then_laplace_threshold()`](https://docs.opendp.org/reference/then_laplace_threshold.md)
  : partial laplace threshold constructor
- [`then_noise()`](https://docs.opendp.org/reference/then_noise.md) :
  partial noise constructor
- [`then_noise_threshold()`](https://docs.opendp.org/reference/then_noise_threshold.md)
  : partial noise threshold constructor
- [`then_noisy_max()`](https://docs.opendp.org/reference/then_noisy_max.md)
  : partial noisy max constructor
- [`then_noisy_top_k()`](https://docs.opendp.org/reference/then_noisy_top_k.md)
  : partial noisy top k constructor
- [`then_private_quantile()`](https://docs.opendp.org/reference/then_private_quantile.md)
  : partial private quantile constructor
- [`then_randomized_response()`](https://docs.opendp.org/reference/then_randomized_response.md)
  : partial randomized response constructor
- [`then_randomized_response_bitvec()`](https://docs.opendp.org/reference/then_randomized_response_bitvec.md)
  : partial randomized response bitvec constructor
- [`then_randomized_response_bool()`](https://docs.opendp.org/reference/then_randomized_response_bool.md)
  : partial randomized response bool constructor
- [`then_report_noisy_max_gumbel()`](https://docs.opendp.org/reference/then_report_noisy_max_gumbel.md)
  : partial report noisy max gumbel constructor

## Measures

The `measures` modules provides functions that measure the distance
between probability distributions.

- [`approximate()`](https://docs.opendp.org/reference/approximate.md) :
  Privacy measure used to define \\(\delta\\)-approximate
  PM-differential privacy.

- [`approximate_divergence_get_inner_measure()`](https://docs.opendp.org/reference/approximate_divergence_get_inner_measure.md)
  : Retrieve the inner privacy measure of an approximate privacy
  measure.

- [`fixed_smoothed_max_divergence()`](https://docs.opendp.org/reference/fixed_smoothed_max_divergence.md)
  : Privacy measure used to define \\((\epsilon, \delta)\\)-approximate
  differential privacy.

- [`max_divergence()`](https://docs.opendp.org/reference/max_divergence.md)
  : Privacy measure used to define \\(\epsilon\\)-pure differential
  privacy.

- [`measure_debug()`](https://docs.opendp.org/reference/measure_debug.md)
  :

  Debug a `measure`.

- [`measure_distance_type()`](https://docs.opendp.org/reference/measure_distance_type.md)
  :

  Get the distance type of a `measure`.

- [`measure_equal()`](https://docs.opendp.org/reference/measure_equal.md)
  : Check whether two measures are equal.

- [`measure_type()`](https://docs.opendp.org/reference/measure_type.md)
  :

  Get the type of a `measure`.

- [`renyi_divergence()`](https://docs.opendp.org/reference/renyi_divergence.md)
  : Privacy measure used to define \\(\epsilon(\alpha)\\)-Rényi
  differential privacy.

- [`smoothed_max_divergence()`](https://docs.opendp.org/reference/smoothed_max_divergence.md)
  : Privacy measure used to define \\(\epsilon(\delta)\\)-approximate
  differential privacy.

- [`user_divergence()`](https://docs.opendp.org/reference/user_divergence.md)
  : Privacy measure with meaning defined by an OpenDP Library user
  (you).

- [`zero_concentrated_divergence()`](https://docs.opendp.org/reference/zero_concentrated_divergence.md)
  : Privacy measure used to define \\(\rho\\)-zero concentrated
  differential privacy.

## Metrics

The `metrics` module provides functions that measure the distance
between two elements of a domain.

- [`absolute_distance()`](https://docs.opendp.org/reference/absolute_distance.md)
  :

  Construct an instance of the `AbsoluteDistance` metric.

- [`change_one_distance()`](https://docs.opendp.org/reference/change_one_distance.md)
  :

  Construct an instance of the `ChangeOneDistance` metric.

- [`discrete_distance()`](https://docs.opendp.org/reference/discrete_distance.md)
  :

  Construct an instance of the `DiscreteDistance` metric.

- [`hamming_distance()`](https://docs.opendp.org/reference/hamming_distance.md)
  :

  Construct an instance of the `HammingDistance` metric.

- [`insert_delete_distance()`](https://docs.opendp.org/reference/insert_delete_distance.md)
  :

  Construct an instance of the `InsertDeleteDistance` metric.

- [`l01inf_distance()`](https://docs.opendp.org/reference/l01inf_distance.md)
  :

  Construct an instance of the `L01InfDistance` metric.

- [`l02inf_distance()`](https://docs.opendp.org/reference/l02inf_distance.md)
  :

  Construct an instance of the `L02InfDistance` metric.

- [`l1_distance()`](https://docs.opendp.org/reference/l1_distance.md) :

  Construct an instance of the `L1Distance` metric.

- [`l2_distance()`](https://docs.opendp.org/reference/l2_distance.md) :

  Construct an instance of the `L2Distance` metric.

- [`linf_distance()`](https://docs.opendp.org/reference/linf_distance.md)
  :

  Construct an instance of the `LInfDistance` metric.

- [`metric_debug()`](https://docs.opendp.org/reference/metric_debug.md)
  :

  Debug a `metric`.

- [`metric_distance_type()`](https://docs.opendp.org/reference/metric_distance_type.md)
  :

  Get the distance type of a `metric`.

- [`metric_equal()`](https://docs.opendp.org/reference/metric_equal.md)
  : Check whether two metrics are equal.

- [`metric_type()`](https://docs.opendp.org/reference/metric_type.md) :

  Get the type of a `metric`.

- [`symmetric_distance()`](https://docs.opendp.org/reference/symmetric_distance.md)
  :

  Construct an instance of the `SymmetricDistance` metric.

## Mod

The `mod` module provides the classes which implement the [OpenDP
Programming
Framework](https://docs.opendp.org/en/stable/api/user-guide/programming-framework/index.html),
as well as utilities for enabling features and finding parameter values.

- [`binary_search()`](https://docs.opendp.org/reference/binary_search.md)
  :

  Find the closest passing value to the decision boundary of `predicate`

- [`binary_search_chain()`](https://docs.opendp.org/reference/binary_search_chain.md)
  :

  Find the highest-utility (`d_in`, `d_out`)-close Transformation or
  Measurement.

- [`binary_search_param()`](https://docs.opendp.org/reference/binary_search_param.md)
  :

  Solve for the ideal constructor argument to `make_chain`

- [`disable_features()`](https://docs.opendp.org/reference/disable_features.md)
  : Disable features in the opendp package.

- [`enable_features()`](https://docs.opendp.org/reference/enable_features.md)
  : Enable features for the opendp package.

- [`hashitems()`](https://docs.opendp.org/reference/hashitems.md) :
  extract heterogeneously typed keys and values from a hashtab

- [`new_domain()`](https://docs.opendp.org/reference/new_domain.md) :
  new domain

- [`new_function()`](https://docs.opendp.org/reference/new_function.md)
  : new function

- [`new_hashtab()`](https://docs.opendp.org/reference/new_hashtab.md) :
  create an instance of a hashtab from keys and values

- [`new_measure()`](https://docs.opendp.org/reference/new_measure.md) :
  new measure

- [`new_measurement()`](https://docs.opendp.org/reference/new_measurement.md)
  : new measurement

- [`new_metric()`](https://docs.opendp.org/reference/new_metric.md) :
  new metric

- [`new_odometer()`](https://docs.opendp.org/reference/new_odometer.md)
  : new odometer

- [`new_odometer_queryable()`](https://docs.opendp.org/reference/new_odometer_queryable.md)
  : new odometer queryable

- [`new_privacy_profile()`](https://docs.opendp.org/reference/new_privacy_profile.md)
  : new privacy profile

- [`new_queryable()`](https://docs.opendp.org/reference/new_queryable.md)
  : new queryable

- [`new_transformation()`](https://docs.opendp.org/reference/new_transformation.md)
  : new transformation

- [`to_str(`*`<default>`*`)`](https://docs.opendp.org/reference/to_str.default.md)
  : Convert a format-able value to a string representation

- [`to_str(`*`<hashtab>`*`)`](https://docs.opendp.org/reference/to_str.hashtab.md)
  : Convert hashtab to a string representation

## Transformations

The `transformations` module provides functions that deterministically
transform datasets.

- [`choose_branching_factor()`](https://docs.opendp.org/reference/choose_branching_factor.md)
  :

  Returns an approximation to the ideal `branching_factor` for a dataset
  of a given size, that minimizes error in cdf and quantile estimates
  based on b-ary trees.

- [`make_b_ary_tree()`](https://docs.opendp.org/reference/make_b_ary_tree.md)
  : b ary tree constructor

- [`make_bounded_float_checked_sum()`](https://docs.opendp.org/reference/make_bounded_float_checked_sum.md)
  : bounded float checked sum constructor

- [`make_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/make_bounded_float_ordered_sum.md)
  : bounded float ordered sum constructor

- [`make_bounded_int_monotonic_sum()`](https://docs.opendp.org/reference/make_bounded_int_monotonic_sum.md)
  : bounded int monotonic sum constructor

- [`make_bounded_int_ordered_sum()`](https://docs.opendp.org/reference/make_bounded_int_ordered_sum.md)
  : bounded int ordered sum constructor

- [`make_bounded_int_split_sum()`](https://docs.opendp.org/reference/make_bounded_int_split_sum.md)
  : bounded int split sum constructor

- [`make_cast()`](https://docs.opendp.org/reference/make_cast.md) : cast
  constructor

- [`make_cast_default()`](https://docs.opendp.org/reference/make_cast_default.md)
  : cast default constructor

- [`make_cast_inherent()`](https://docs.opendp.org/reference/make_cast_inherent.md)
  : cast inherent constructor

- [`make_cdf()`](https://docs.opendp.org/reference/make_cdf.md) : cdf
  constructor

- [`make_clamp()`](https://docs.opendp.org/reference/make_clamp.md) :
  clamp constructor

- [`make_consistent_b_ary_tree()`](https://docs.opendp.org/reference/make_consistent_b_ary_tree.md)
  : consistent b ary tree constructor

- [`make_count()`](https://docs.opendp.org/reference/make_count.md) :
  count constructor

- [`make_count_by()`](https://docs.opendp.org/reference/make_count_by.md)
  : count by constructor

- [`make_count_by_categories()`](https://docs.opendp.org/reference/make_count_by_categories.md)
  : count by categories constructor

- [`make_count_distinct()`](https://docs.opendp.org/reference/make_count_distinct.md)
  : count distinct constructor

- [`make_create_dataframe()`](https://docs.opendp.org/reference/make_create_dataframe.md)
  : create dataframe constructor

- [`make_df_cast_default()`](https://docs.opendp.org/reference/make_df_cast_default.md)
  : df cast default constructor

- [`make_df_is_equal()`](https://docs.opendp.org/reference/make_df_is_equal.md)
  : df is equal constructor

- [`make_drop_null()`](https://docs.opendp.org/reference/make_drop_null.md)
  : drop null constructor

- [`make_find()`](https://docs.opendp.org/reference/make_find.md) : find
  constructor

- [`make_find_bin()`](https://docs.opendp.org/reference/make_find_bin.md)
  : find bin constructor

- [`make_identity()`](https://docs.opendp.org/reference/make_identity.md)
  : identity constructor

- [`make_impute_constant()`](https://docs.opendp.org/reference/make_impute_constant.md)
  : impute constant constructor

- [`make_impute_uniform_float()`](https://docs.opendp.org/reference/make_impute_uniform_float.md)
  : impute uniform float constructor

- [`make_index()`](https://docs.opendp.org/reference/make_index.md) :
  index constructor

- [`make_is_equal()`](https://docs.opendp.org/reference/make_is_equal.md)
  : is equal constructor

- [`make_is_null()`](https://docs.opendp.org/reference/make_is_null.md)
  : is null constructor

- [`make_lipschitz_float_mul()`](https://docs.opendp.org/reference/make_lipschitz_float_mul.md)
  : lipschitz float mul constructor

- [`make_mean()`](https://docs.opendp.org/reference/make_mean.md) : mean
  constructor

- [`make_metric_bounded()`](https://docs.opendp.org/reference/make_metric_bounded.md)
  : metric bounded constructor

- [`make_metric_unbounded()`](https://docs.opendp.org/reference/make_metric_unbounded.md)
  : metric unbounded constructor

- [`make_ordered_random()`](https://docs.opendp.org/reference/make_ordered_random.md)
  : ordered random constructor

- [`make_quantile_score_candidates()`](https://docs.opendp.org/reference/make_quantile_score_candidates.md)
  : quantile score candidates constructor

- [`make_quantiles_from_counts()`](https://docs.opendp.org/reference/make_quantiles_from_counts.md)
  : quantiles from counts constructor

- [`make_resize()`](https://docs.opendp.org/reference/make_resize.md) :
  resize constructor

- [`make_select_column()`](https://docs.opendp.org/reference/make_select_column.md)
  : select column constructor

- [`make_sized_bounded_float_checked_sum()`](https://docs.opendp.org/reference/make_sized_bounded_float_checked_sum.md)
  : sized bounded float checked sum constructor

- [`make_sized_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/make_sized_bounded_float_ordered_sum.md)
  : sized bounded float ordered sum constructor

- [`make_sized_bounded_int_checked_sum()`](https://docs.opendp.org/reference/make_sized_bounded_int_checked_sum.md)
  : sized bounded int checked sum constructor

- [`make_sized_bounded_int_monotonic_sum()`](https://docs.opendp.org/reference/make_sized_bounded_int_monotonic_sum.md)
  : sized bounded int monotonic sum constructor

- [`make_sized_bounded_int_ordered_sum()`](https://docs.opendp.org/reference/make_sized_bounded_int_ordered_sum.md)
  : sized bounded int ordered sum constructor

- [`make_sized_bounded_int_split_sum()`](https://docs.opendp.org/reference/make_sized_bounded_int_split_sum.md)
  : sized bounded int split sum constructor

- [`make_split_dataframe()`](https://docs.opendp.org/reference/make_split_dataframe.md)
  : split dataframe constructor

- [`make_split_lines()`](https://docs.opendp.org/reference/make_split_lines.md)
  : split lines constructor

- [`make_split_records()`](https://docs.opendp.org/reference/make_split_records.md)
  : split records constructor

- [`make_subset_by()`](https://docs.opendp.org/reference/make_subset_by.md)
  : subset by constructor

- [`make_sum()`](https://docs.opendp.org/reference/make_sum.md) : sum
  constructor

- [`make_sum_of_squared_deviations()`](https://docs.opendp.org/reference/make_sum_of_squared_deviations.md)
  : sum of squared deviations constructor

- [`make_unordered()`](https://docs.opendp.org/reference/make_unordered.md)
  : unordered constructor

- [`make_variance()`](https://docs.opendp.org/reference/make_variance.md)
  : variance constructor

- [`then_b_ary_tree()`](https://docs.opendp.org/reference/then_b_ary_tree.md)
  : partial b ary tree constructor

- [`then_bounded_float_checked_sum()`](https://docs.opendp.org/reference/then_bounded_float_checked_sum.md)
  : partial bounded float checked sum constructor

- [`then_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/then_bounded_float_ordered_sum.md)
  : partial bounded float ordered sum constructor

- [`then_bounded_int_monotonic_sum()`](https://docs.opendp.org/reference/then_bounded_int_monotonic_sum.md)
  : partial bounded int monotonic sum constructor

- [`then_bounded_int_ordered_sum()`](https://docs.opendp.org/reference/then_bounded_int_ordered_sum.md)
  : partial bounded int ordered sum constructor

- [`then_bounded_int_split_sum()`](https://docs.opendp.org/reference/then_bounded_int_split_sum.md)
  : partial bounded int split sum constructor

- [`then_cast()`](https://docs.opendp.org/reference/then_cast.md) :
  partial cast constructor

- [`then_cast_default()`](https://docs.opendp.org/reference/then_cast_default.md)
  : partial cast default constructor

- [`then_cast_inherent()`](https://docs.opendp.org/reference/then_cast_inherent.md)
  : partial cast inherent constructor

- [`then_cdf()`](https://docs.opendp.org/reference/then_cdf.md) :
  partial cdf constructor

- [`then_clamp()`](https://docs.opendp.org/reference/then_clamp.md) :
  partial clamp constructor

- [`then_consistent_b_ary_tree()`](https://docs.opendp.org/reference/then_consistent_b_ary_tree.md)
  : partial consistent b ary tree constructor

- [`then_count()`](https://docs.opendp.org/reference/then_count.md) :
  partial count constructor

- [`then_count_by()`](https://docs.opendp.org/reference/then_count_by.md)
  : partial count by constructor

- [`then_count_by_categories()`](https://docs.opendp.org/reference/then_count_by_categories.md)
  : partial count by categories constructor

- [`then_count_distinct()`](https://docs.opendp.org/reference/then_count_distinct.md)
  : partial count distinct constructor

- [`then_create_dataframe()`](https://docs.opendp.org/reference/then_create_dataframe.md)
  : partial create dataframe constructor

- [`then_df_cast_default()`](https://docs.opendp.org/reference/then_df_cast_default.md)
  : partial df cast default constructor

- [`then_df_is_equal()`](https://docs.opendp.org/reference/then_df_is_equal.md)
  : partial df is equal constructor

- [`then_drop_null()`](https://docs.opendp.org/reference/then_drop_null.md)
  : partial drop null constructor

- [`then_find()`](https://docs.opendp.org/reference/then_find.md) :
  partial find constructor

- [`then_find_bin()`](https://docs.opendp.org/reference/then_find_bin.md)
  : partial find bin constructor

- [`then_identity()`](https://docs.opendp.org/reference/then_identity.md)
  : partial identity constructor

- [`then_impute_constant()`](https://docs.opendp.org/reference/then_impute_constant.md)
  : partial impute constant constructor

- [`then_impute_uniform_float()`](https://docs.opendp.org/reference/then_impute_uniform_float.md)
  : partial impute uniform float constructor

- [`then_index()`](https://docs.opendp.org/reference/then_index.md) :
  partial index constructor

- [`then_is_equal()`](https://docs.opendp.org/reference/then_is_equal.md)
  : partial is equal constructor

- [`then_is_null()`](https://docs.opendp.org/reference/then_is_null.md)
  : partial is null constructor

- [`then_lipschitz_float_mul()`](https://docs.opendp.org/reference/then_lipschitz_float_mul.md)
  : partial lipschitz float mul constructor

- [`then_mean()`](https://docs.opendp.org/reference/then_mean.md) :
  partial mean constructor

- [`then_metric_bounded()`](https://docs.opendp.org/reference/then_metric_bounded.md)
  : partial metric bounded constructor

- [`then_metric_unbounded()`](https://docs.opendp.org/reference/then_metric_unbounded.md)
  : partial metric unbounded constructor

- [`then_ordered_random()`](https://docs.opendp.org/reference/then_ordered_random.md)
  : partial ordered random constructor

- [`then_quantile_score_candidates()`](https://docs.opendp.org/reference/then_quantile_score_candidates.md)
  : partial quantile score candidates constructor

- [`then_quantiles_from_counts()`](https://docs.opendp.org/reference/then_quantiles_from_counts.md)
  : partial quantiles from counts constructor

- [`then_resize()`](https://docs.opendp.org/reference/then_resize.md) :
  partial resize constructor

- [`then_select_column()`](https://docs.opendp.org/reference/then_select_column.md)
  : partial select column constructor

- [`then_sized_bounded_float_checked_sum()`](https://docs.opendp.org/reference/then_sized_bounded_float_checked_sum.md)
  : partial sized bounded float checked sum constructor

- [`then_sized_bounded_float_ordered_sum()`](https://docs.opendp.org/reference/then_sized_bounded_float_ordered_sum.md)
  : partial sized bounded float ordered sum constructor

- [`then_sized_bounded_int_checked_sum()`](https://docs.opendp.org/reference/then_sized_bounded_int_checked_sum.md)
  : partial sized bounded int checked sum constructor

- [`then_sized_bounded_int_monotonic_sum()`](https://docs.opendp.org/reference/then_sized_bounded_int_monotonic_sum.md)
  : partial sized bounded int monotonic sum constructor

- [`then_sized_bounded_int_ordered_sum()`](https://docs.opendp.org/reference/then_sized_bounded_int_ordered_sum.md)
  : partial sized bounded int ordered sum constructor

- [`then_sized_bounded_int_split_sum()`](https://docs.opendp.org/reference/then_sized_bounded_int_split_sum.md)
  : partial sized bounded int split sum constructor

- [`then_split_dataframe()`](https://docs.opendp.org/reference/then_split_dataframe.md)
  : partial split dataframe constructor

- [`then_split_lines()`](https://docs.opendp.org/reference/then_split_lines.md)
  : partial split lines constructor

- [`then_split_records()`](https://docs.opendp.org/reference/then_split_records.md)
  : partial split records constructor

- [`then_subset_by()`](https://docs.opendp.org/reference/then_subset_by.md)
  : partial subset by constructor

- [`then_sum()`](https://docs.opendp.org/reference/then_sum.md) :
  partial sum constructor

- [`then_sum_of_squared_deviations()`](https://docs.opendp.org/reference/then_sum_of_squared_deviations.md)
  : partial sum of squared deviations constructor

- [`then_unordered()`](https://docs.opendp.org/reference/then_unordered.md)
  : partial unordered constructor

- [`then_variance()`](https://docs.opendp.org/reference/then_variance.md)
  : partial variance constructor

## Typing

The `typing` module provides utilities that bridge between R and Rust
types. OpenDP relies on precise descriptions of data types to make its
security guarantees: These are more natural in Rust with its
fine-grained type system, but they may feel out of place in R. These
utilities try to fill that gap.

- [`BitVector`](https://docs.opendp.org/reference/BitVector.md) : type
  signature for a BitVector
- [`String`](https://docs.opendp.org/reference/String.md) : type
  signature for a string
- [`bool`](https://docs.opendp.org/reference/bool.md) : type signature
  for a boolean
- [`f32`](https://docs.opendp.org/reference/f32.md) : type signature for
  a 32-bit floating point number
- [`f64`](https://docs.opendp.org/reference/f64.md) : type signature for
  a 64-bit floating point number
- [`i128`](https://docs.opendp.org/reference/i128.md) : type signature
  for a 128-bit signed integer
- [`i16`](https://docs.opendp.org/reference/i16.md) : type signature for
  a 16-bit signed integer
- [`i32`](https://docs.opendp.org/reference/i32.md) : type signature for
  a 32-bit signed integer
- [`i64`](https://docs.opendp.org/reference/i64.md) : type signature for
  a 64-bit signed integer
- [`i8`](https://docs.opendp.org/reference/i8.md) : type signature for
  an 8-bit signed integer
- [`u128`](https://docs.opendp.org/reference/u128.md) : type signature
  for a 128-bit unsigned integer
- [`u16`](https://docs.opendp.org/reference/u16.md) : type signature for
  a 16-bit unsigned integer
- [`u32`](https://docs.opendp.org/reference/u32.md) : type signature for
  a 32-bit unsigned integer
- [`u64`](https://docs.opendp.org/reference/u64.md) : type signature for
  a 64-bit unsigned integer
- [`u8`](https://docs.opendp.org/reference/u8.md) : type signature for
  an 8-bit unsigned integer
- [`usize`](https://docs.opendp.org/reference/usize.md) : type signature
  for a pointer-sized unsigned integer

## Other

This should be empty if correctly configured. Please file an issue if
any functions are listed here.
