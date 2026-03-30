from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

import opendp.prelude as dp
from opendp._internal import _make_transformation, _new_pure_function
from opendp.extras.numpy._make_np_clamp import make_np_clamp
from opendp.extras.numpy._make_np_count import make_np_count
from opendp.extras.numpy._make_np_sum import make_np_sum
from opendp.extras.sklearn.cluster._tree import (
    _budget_with_slack,
    _budget_without_slack,
    _noise_scale_for_budget,
    _noise_var,
    make_parallel_composition,
    parallel_distance,
)

if TYPE_CHECKING:  # pragma: no cover
    import numpy


@dataclass
class ClusterStats:
    count: float
    sum: "numpy.ndarray"
    count_var: float
    sum_var: float


@dataclass
class LloydRefinementResult:
    centers: "numpy.ndarray"
    counts: "numpy.ndarray"
    iters: int


def _np():
    from opendp._lib import import_optional_dependency

    return import_optional_dependency("numpy")


def make_private_lloyd_iteration(
    input_domain,
    input_metric,
    output_measure,
    *,
    centers,
    lower,
    upper,
    count_budget: float,
    sum_budget: float,
):
    np = _np()
    centers = np.asarray(centers, dtype=float)
    lower = np.asarray(lower, dtype=float)
    upper = np.asarray(upper, dtype=float)
    if centers.ndim != 2:
        raise ValueError("centers must be a 2D array")
    if len(centers) == 0:
        raise ValueError("centers must be nonempty")

    k, d = centers.shape
    origin = 0.5 * (lower + upper)
    p = 1 if output_measure == dp.max_divergence() else 2
    norm = 0.5 * float(np.linalg.norm(upper - lower, ord=p))
    if norm <= 0.0:
        raise ValueError("refinement requires a nondegenerate public bounding box")

    def assign_to_clusters(data):
        data = np.asarray(data)
        if data.ndim != 2:
            data = data.reshape((-1, d))
        diff = data[:, None, :] - centers[None, :, :]
        labels = np.argmin(np.sum(diff * diff, axis=2), axis=1)
        return [data[labels == idx] for idx in range(k)]

    t_assign = _make_transformation(
        input_domain=input_domain,
        input_metric=input_metric,
        output_domain=dp.vector_domain(input_domain, size=k),
        output_metric=parallel_distance(input_metric),
        function=assign_to_clusters,
        stability_map=lambda d_in: (1, d_in),
    )

    count_scale = _noise_scale_for_budget(
        output_measure,
        _budget_without_slack(count_budget),
    )
    t_count = make_np_count(input_domain, input_metric)
    m_count = t_count >> dp.m.then_noise(output_measure, count_scale)

    t_clamp = make_np_clamp(
        input_domain,
        input_metric,
        norm=norm,
        p=p,
        origin=origin,
    )
    t_sum = make_np_sum(t_clamp.output_domain, t_clamp.output_metric)
    sum_stability = float(t_sum.map(1))
    sum_scale = sum_stability * _noise_scale_for_budget(
        output_measure,
        _budget_without_slack(sum_budget),
    )
    m_sum = t_clamp >> t_sum >> dp.m.then_noise(output_measure, sum_scale)

    m_cluster = dp.c.make_composition([m_count, m_sum]) >> _new_pure_function(
        lambda outputs: ClusterStats(
            count=float(outputs[0]),
            sum=np.asarray(outputs[1], dtype=float),
            count_var=_noise_var(output_measure, count_scale),
            sum_var=_noise_var(output_measure, sum_scale),
        ),
        TO="ExtrinsicObject",
    )
    return t_assign >> make_parallel_composition([m_cluster] * k)


def update_centers_from_private_stats(
    centers,
    stats: list[ClusterStats],
    *,
    lower,
    upper,
    refinement_min_count: float | str,
):
    np = _np()
    centers = np.asarray(centers, dtype=float)
    lower = np.asarray(lower, dtype=float)
    upper = np.asarray(upper, dtype=float)
    updated = centers.copy()
    for idx, stat in enumerate(stats):
        min_count = (
            float(np.sqrt(stat.count_var))
            if refinement_min_count == "auto"
            else float(refinement_min_count)
        )
        if stat.count > min_count:
            updated[idx] = np.clip(stat.sum / stat.count, lower, upper)
    return updated


def make_private_lloyd_refinement(
    input_domain,
    input_metric,
    output_measure,
    *,
    initial_centers,
    lower,
    upper,
    refinement_iters: int,
    count_budget: float,
    sum_budget: float,
    refinement_min_count: float | str,
):
    np = _np()
    initial_centers = np.asarray(initial_centers, dtype=float)
    if refinement_iters <= 0:
        raise ValueError("refinement_iters must be positive")

    m_iter_prototype = make_private_lloyd_iteration(
        input_domain,
        input_metric,
        output_measure,
        centers=initial_centers,
        lower=lower,
        upper=upper,
        count_budget=count_budget,
        sum_budget=sum_budget,
    )
    iter_budget = _budget_with_slack(m_iter_prototype.map(1))
    comp = dp.c.make_adaptive_composition(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        d_in=1,
        d_mids=[iter_budget] * refinement_iters,
    )

    def postprocess(qbl):
        centers = np.asarray(initial_centers, dtype=float)
        counts = np.zeros(len(centers), dtype=float)
        for _ in range(refinement_iters):
            m_iter = make_private_lloyd_iteration(
                input_domain,
                input_metric,
                output_measure,
                centers=centers,
                lower=lower,
                upper=upper,
                count_budget=count_budget,
                sum_budget=sum_budget,
            )
            stats = qbl(m_iter)
            counts = np.asarray([stat.count for stat in stats], dtype=float)
            centers = update_centers_from_private_stats(
                centers,
                stats,
                lower=lower,
                upper=upper,
                refinement_min_count=refinement_min_count,
            )
        return LloydRefinementResult(
            centers=centers,
            counts=counts,
            iters=refinement_iters,
        )

    return comp >> _new_pure_function(postprocess, TO="ExtrinsicObject")
