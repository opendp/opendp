import math
from itertools import chain, combinations
from typing import Optional

from opendp._lib import import_optional_dependency
from opendp.combinators import (
    make_privacy_filter,
    make_pureDP_to_zCDP,
    then_fully_adaptive_composition,
)
from opendp.domains import atom_domain, vector_domain
from opendp.extras.synthetic.domains import linf_l2_distance, tuple_domain
from opendp.measurements import (
    then_gaussian,
    then_report_noisy_max_gumbel,
)
from opendp.measures import zero_concentrated_divergence
from opendp.metrics import l1_distance, l2_distance, linf_distance, symmetric_distance
from opendp.mod import (
    ExtrinsicDomain,
    Measure,
    Metric,
    binary_search_chain,
    binary_search_param,
)
from opendp._internal import (
    _make_measurement,
    _make_transformation,
    _new_pure_function,
)
from opendp.extras.numpy import NPArray2Domain, NPArrayDDomain, arrayd_domain

from typing import TYPE_CHECKING

if TYPE_CHECKING: # pragma: no cover
    from mbi import LinearMeasurement, MarkovRandomField  # type: ignore


def make_ordinal_aim(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    output_measure: Measure,
    d_in: int,
    d_out: float,
    releases: list["LinearMeasurement"],
    queries: list[list[int]],
    weights: Optional[list[float]] = None,
    max_size: int = 80,
    max_iters: int = 100,
    alpha: float = 0.9,
):
    """
    Implements AIM (Adaptive Iterative Mechanism) for ordinal data.

    :param input_domain: The domain of the input data
    :param input_metric: The metric for the input domain
    :param output_measure: The privacy measure
    :param d_in: distance between adjacent input datasets
    :param d_out: upper bound on the privacy loss
    :param queries: Workload of queries, each query is a list of column indices
    :param weights: Weights for each query in the workload
    :param max_size: Maximum memory constraint for the marginal selection
    :param max_iters: Maximum number of iterations for the mirror descent algorithm
    :param alpha: Parameter used to balance noise scales for different mechanisms
    :param releases: List of LinearMeasurement objects for previously released data
    """
    np = import_optional_dependency("numpy")
    mbi = import_optional_dependency("mbi")

    if not isinstance(input_domain.descriptor, NPArray2Domain):
        raise ValueError("input_domain must be opendp.extras.numpy.array2_domain")

    cardinalities = input_domain.descriptor.cardinalities
    if cardinalities is None:
        raise ValueError("input_domain cardinalities must be defined")

    if input_metric != symmetric_distance():
        raise ValueError("input_metric must be symmetric_distance")

    if output_measure != zero_concentrated_divergence():
        raise ValueError("output_measure must be zero_concentrated_divergence")

    if weights is None:  # default to equal weighting
        weights = [1.0] * len(queries)

    if len(weights) != len(queries):
        raise ValueError("weights must have the same length as queries")

    if any(w < 0 for w in weights):
        raise ValueError("weights must be non-negative")

    queries, weights = expand_queries(queries, weights)

    mbi_domain = mbi.Domain(
        attributes=tuple(range(len(cardinalities))), shape=cardinalities
    )

    t_crosstabs = make_all_crosstabs(input_domain, input_metric, queries)

    m_filter = make_privacy_filter(
        t_crosstabs >> then_fully_adaptive_composition(output_measure),
        d_in=d_in,
        d_out=d_out,
    )

    # Initialize noise parameters
    T = 16 * cardinalities.size

    # Define the AIM mechanism
    def function(data):
        # all access to data mediated by a privacy filter
        qbl = m_filter(data)
        del data

        selected_indices: list[int] = []
        current_releases = releases.copy()

        # Initialize budget parameters
        d_select = d_out * (1 - alpha) / T  # i.e. epsilon**2 / 8
        d_measure = d_out * alpha / T  # i.e. 1/(2 * sigma**2)

        model = mbi.estimation.mirror_descent(
            mbi_domain, current_releases, iters=max_iters, callback_fn=lambda *_: None
        )

        d_current = 0
        while d_current < d_out - 1e-9:
            # SELECT a query that best reduces the error
            selected_measurement = make_select( # type: ignore[misc]
                *t_crosstabs.output_space,
                queries=queries,
                weights=weights,
                selected_indices=selected_indices,
                model=model,
                d_in=d_in,
                d_out=d_select,
                max_size=max_size,
                mbi_domain=mbi_domain,
            )

            if selected_measurement is None:
                break

            selected_index = qbl(selected_measurement)
            selected_indices.append(selected_index)
            selected_query = queries[selected_index]

            d_current = qbl.privacy_loss(d_in)

            # MEASURE selected marginal with noise
            m_measure, scale = make_measure( # type: ignore[call-arg,misc]
                *t_crosstabs.output_space,
                selected_index,
                d_in=d_in,
                d_out=d_measure,
            )

            current_releases.append(
                mbi.LinearMeasurement(
                    noisy_measurement=qbl(m_measure).flatten(),
                    clique=selected_query,
                    stddev=scale,
                )
            )

            # GENERATE an updated probability distribution
            prev_tab = model.project(selected_query).datavector()

            pcliques = list(set(M.clique for M in current_releases))
            model = mbi.estimation.mirror_descent(
                mbi_domain,
                current_releases,
                iters=max_iters,
                potentials=model.potentials.expand(pcliques),
                callback_fn=lambda *_: None,
            )

            next_tab = model.project(selected_query).datavector()
            error = np.linalg.norm(next_tab - prev_tab, 1)
            size = np.prod(cardinalities[list(selected_query)])

            if error <= scale * np.sqrt(2 / np.pi) * size:
                d_select *= 4
                d_measure *= 4

            d_current = qbl.privacy_loss(d_in)
            d_remaining = np.nextafter(d_out - d_current, -np.inf)

            if np.nextafter(d_select + d_measure, np.inf) > d_remaining:
                d_select = np.nextafter(d_remaining * (1 - alpha), -np.inf)
                d_measure = np.nextafter(d_remaining * alpha, -np.inf)

        #     print(f"Privacy used so far: rho = {d_current}")
        # print(f"Total privacy budget expended: rho = {d_current}")

        return current_releases

    return _make_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=function,
        privacy_map=lambda d_in_p: d_in_p // d_in * d_out,
    )


def expand_queries(queries, weights):
    """Precomputes workload information including W_plus, marginal weights, and n_r_array."""
    expanded_queries = list(
        set(chain.from_iterable(get_all_subsets(query) for query in queries))
    )

    def compute_weight(new_query):
        return sum(
            weight * len(set(new_query) & set(query))
            for query, weight in zip(queries, weights)
        )

    marginal_weights = [compute_weight(query) for query in expanded_queries]

    return expanded_queries, marginal_weights


def make_all_crosstabs(input_domain, input_metric, W_plus):
    """Computes all marginals for the workload W_plus."""
    t_crosstabs = [
        make_stable_crosstab(input_domain, input_metric, query) for query in W_plus
    ]

    return _make_transformation(
        input_domain,
        input_metric,
        output_domain=tuple_domain(tuple(t.output_domain for t in t_crosstabs)),
        output_metric=linf_l2_distance(),
        function=lambda data: tuple(t_crosstab(data) for t_crosstab in t_crosstabs),
        stability_map=lambda d_in: d_in,
    )


def make_select(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    d_in,
    d_out,
    queries,
    weights,
    selected_indices,
    model: "MarkovRandomField",
    max_size,
    mbi_domain,
):
    # only keep queries that fit the memory constraint
    valid_indices = [
        index
        for index, query in enumerate(queries)
        if is_memory_bounded(
            query, d_in, d_out, max_size, mbi_domain, selected_indices, queries
        )
        and index not in selected_indices
    ]

    if not valid_indices:
        return None

    queries = [queries[i] for i in valid_indices]
    weights = [weights[i] for i in valid_indices]

    return binary_search_chain(
        lambda scale: make_pureDP_to_zCDP(
            make_score_crosstabs(
                input_domain,
                input_metric,
                queries,
                weights,
                selected_indices,
                model,
                scale,
            )
            >> then_report_noisy_max_gumbel(scale=scale, optimize="max")
        ),
        d_in=d_in,
        d_out=d_out,
        T=float,
    ) >> _new_pure_function(lambda idx: valid_indices[idx], TO=int)


def make_score_crosstabs(
    input_domain: ExtrinsicDomain,
    input_metric: Metric,
    queries,
    weights,
    selected_indices,
    model: "MarkovRandomField",
    scale,
):
    np = import_optional_dependency("numpy")

    if not str(input_domain).startswith("TupleDomain"):
        raise ValueError(
            "input_domain must be opendp.synthetic.aim.tuple_domain"
        )  # pragma: no cover

    if not all(
        isinstance(d.descriptor, NPArrayDDomain) for d in input_domain.descriptor
    ):
        raise ValueError(  # pragma: no cover
            "input_domain elements must be opendp.extras.numpy.arrayd_domain"
        )

    penalties = [
        math.sqrt(2 / math.pi) * scale * np.prod(d.descriptor.shape)
        for d in input_domain.descriptor
    ]
    synth_tabs = [model.project(query).datavector() for query in queries]

    def function(exact_tabs):
        exact_tabs = [
            exact_tabs[i] for i in range(len(exact_tabs)) if i not in selected_indices
        ]
        return [
            score_query(*args)
            for args in zip(exact_tabs, synth_tabs, penalties, weights)
        ]

    def score_query(exact_i, synth_i, penalty_i, weight_i):
        return (np.linalg.norm(exact_i - synth_i, 1) - penalty_i) * weight_i

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=vector_domain(atom_domain(T=float, nan=False)),
        output_metric=linf_distance(float, monotonic=True),
        function=function,
        stability_map=lambda d_in: d_in * max(weights),
    )


def make_stable_crosstab(input_domain, input_metric, query: list[int]):
    np = import_optional_dependency("numpy")
    if not isinstance(input_domain.descriptor, NPArray2Domain):  # pragma: no cover
        raise ValueError("input_domain must be opendp.extras.numpy.array2_domain")

    if input_domain.descriptor.cardinalities is None:  # pragma: no cover
        raise ValueError("input_domain must have known cardinalities")

    if input_metric != symmetric_distance():  # pragma: no cover
        raise ValueError("input_metric must be symmetric_distance")

    cardinalities = input_domain.descriptor.cardinalities[list(query)]

    def function(data):
        flat_indices = np.ravel_multi_index(data[:, query].T, cardinalities)
        return np.bincount(flat_indices, minlength=np.prod(cardinalities))

    return _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=arrayd_domain(shape=tuple(cardinalities.tolist()), T="i64"),
        output_metric=l1_distance("i32"),
        function=function,
        stability_map=lambda d_in: d_in,
    )


def make_measure(input_domain: ExtrinsicDomain, input_metric, index, d_in, d_out):
    np = import_optional_dependency("numpy")

    if not str(input_domain).startswith("TupleDomain"):
        raise ValueError( # pragma: no cover
            "input_domain must be opendp.extras.synthetic.domains.tuple_domain"
        )

    descriptor = input_domain.descriptor
    indexed_domain = descriptor[index]
    if not isinstance(indexed_domain.descriptor, NPArrayDDomain):
        raise ValueError( # pragma: no cover
            "input_domain must be opendp.extras.numpy.domains.arrayd_domain"
        )

    if input_metric != linf_l2_distance(): # pragma: no cover
        raise ValueError("input_metric must be linf_l2_distance")

    t_crosstab = _make_transformation(
        input_domain,
        input_metric,
        output_domain=vector_domain(atom_domain(T="i32")),
        output_metric=l2_distance(T="f64"),
        function=lambda exact_tabs: exact_tabs[index].astype(np.int32),
        stability_map=lambda d_in: d_in,
    )

    scale = binary_search_param(
        lambda scale: t_crosstab.output_space >> then_gaussian(scale),
        d_in=d_in,
        d_out=d_out,
    )

    return (
        t_crosstab
        >> then_gaussian(scale)
        >> _new_pure_function(
            lambda x: np.reshape(x, indexed_domain.descriptor.shape),
            TO="ExtrinsicObject",
        )
    ), scale


def is_memory_bounded(
    query, d_mid, d_out, max_size, mbi_domain, selected_indices, queries
):
    """
    should calculate the junction tree size of this list, then compare it to d_mid/d_out * max_size
    :param query: a query, i.e. list of column indices
    :param d_mid: current privacy lost exhausted
    :param d_out: total privacy budget
    :param max_size: max memory constraint
    :param mbi_domain: MBI domain object
    :param selected_indices: these are indices into the queries array of already selected queries, i.e. list of ints
    :param queries: total list of queries; list of lists of ints
    """
    mbi = import_optional_dependency("mbi")

    current_query_cliques = [
        tuple(str(i) for i in queries[j]) for j in selected_indices
    ]
    new_query_clique = tuple(str(i) for i in query)
    total_cliques = current_query_cliques + [new_query_clique]

    # if the new query were added, this is the size of the junction tree
    hypothetical_size_mb = mbi.junction_tree.hypothetical_model_size(
        mbi_domain, total_cliques
    )
    # space threshold; new query cannot push memory above this value
    maximum_size_mb = d_mid / d_out * max_size
    return hypothetical_size_mb <= maximum_size_mb


def get_all_subsets(x: list):
    return chain.from_iterable(combinations(x, r) for r in range(1, len(x) + 1))
