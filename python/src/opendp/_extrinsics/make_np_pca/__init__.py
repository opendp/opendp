from __future__ import annotations
from typing import NamedTuple, List, Optional

from opendp._extrinsics.make_np_clamp import then_np_clamp
from opendp._extrinsics._utilities import register_measurement, to_then
from opendp._extrinsics._make_np_mean import make_private_np_mean
from opendp._extrinsics._make_np_eigendecomposition import (
    then_private_np_eigendecomposition,
)
from opendp.mod import Domain, Metric, Measurement


class PCAEpsilons(NamedTuple):
    eigvals: float
    eigvecs: List[float]
    mean: Optional[float]


def make_private_np_pca(
    input_domain: Domain,
    input_metric: Metric,
    unit_epsilon: float | PCAEpsilons,
    norm: float | None = None,
    num_components=None,
) -> Measurement:
    """Construct a Measurement that returns the data mean, singular values and right singular vectors.

    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param unit_epsilon: ε-expenditure per changed record in the input data
    :param norm: clamp each row to this norm bound
    :param num_components: optional, number of eigenvectors to release. defaults to num_columns from input_domain

    :returns a Measurement that computes a tuple of (mean, S, Vt)
    """
    import opendp.prelude as dp
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    class PCAResult(NamedTuple):
        mean: np.ndarray
        S: np.ndarray
        Vt: np.ndarray

    input_desc = input_domain.descriptor
    if input_desc.size is None:
        raise ValueError("input_domain's size must be known")

    if input_desc.num_columns is None:
        raise ValueError("input_domain's num_columns must be known")

    if input_desc.p not in {None, 2}:
        raise ValueError("input_domain's norm must be an L2 norm")

    if input_desc.num_columns < 1:
        raise ValueError("input_domain's num_columns must be >= 1")

    num_components = (
        input_desc.num_columns if num_components is None else num_components
    )

    if isinstance(unit_epsilon, float):
        num_eigvec_releases = min(num_components, input_desc.num_columns - 1)
        unit_epsilon = _split_pca_epsilon_evenly(
            unit_epsilon, num_eigvec_releases, estimate_mean=input_desc.origin is None
        )

    if not isinstance(unit_epsilon, PCAEpsilons):
        raise ValueError("epsilon must be a float or instance of PCAEpsilons")

    eigvals_epsilon, eigvecs_epsilons, mean_epsilon = unit_epsilon

    def eig_to_SVt(decomp):
        eigvals, eigvecs = decomp
        return np.sqrt(np.maximum(eigvals, 0))[::-1], eigvecs.T

    def make_eigdecomp(norm, origin):
        return (
            (input_domain, input_metric)
            >> then_np_clamp(norm, p=2, origin=origin)
            >> then_center()
            >> then_private_np_eigendecomposition(eigvals_epsilon, eigvecs_epsilons)
            >> (lambda out: PCAResult(origin, *eig_to_SVt(out)))
        )

    if input_desc.norm is not None:
        if mean_epsilon is not None:
            raise ValueError("mean_epsilon should be zero because origin is known")
        norm = input_desc.norm if norm is None else norm
        norm = min(input_desc.norm, norm)
        return make_eigdecomp(norm, input_desc.origin)
    elif norm is None:
        raise ValueError("must have either bounded `input_domain` or specify `norm`")


    # make releases under the assumption that d_in is 2.
    unit_d_in = 2

    compositor = dp.c.make_sequential_composition(
        input_domain,
        input_metric,
        dp.max_divergence(T=input_desc.T),
        d_in=unit_d_in,
        d_mids=[mean_epsilon, make_eigdecomp(norm, 0).map(unit_d_in)],
    )

    def function(data):
        nonlocal input_domain
        qbl = compositor(data)

        # find origin
        m_mean = dp.binary_search_chain(  # type: ignore[misc]
            lambda s: make_private_np_mean(
                input_domain, input_metric, s, norm=norm, p=1
            ),
            d_in=unit_d_in,
            d_out=mean_epsilon,
            T=float,
        )
        origin = qbl(m_mean)
        # make full release
        return qbl(make_eigdecomp(norm, origin))

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        compositor.output_measure,
        function,
        compositor.map,
    )


# generate then variant of the constructor
then_private_np_pca = register_measurement(make_private_np_pca)


def _smaller(v):
    """returns the next non-negative float closer to zero"""
    import numpy as np

    if v < 0:
        raise ValueError("expected non-negative value")
    return v if v == 0 else np.nextafter(v, -1)


def _split_pca_epsilon_evenly(unit_epsilon, num_eigvec_releases, estimate_mean=False):
    num_queries = 3 if estimate_mean else 2
    per_query_epsilon = unit_epsilon / num_queries
    per_evec_epsilon = per_query_epsilon / num_eigvec_releases

    # use conservatively smaller budgets to prevent totals from exceeding total epsilon
    return PCAEpsilons(
        eigvals=per_query_epsilon,
        eigvecs=[_smaller(per_evec_epsilon)] * num_eigvec_releases,
        mean=_smaller(per_query_epsilon) if estimate_mean else None,
    )


def _make_center(input_domain, input_metric):
    import opendp.prelude as dp
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    input_desc = input_domain.descriptor

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.np_array2_domain(
            **{
                **input_desc._asdict(),
                "origin": np.zeros(input_desc.num_columns),
            }  # type: ignore[arg-type]
        ),
        input_metric,
        lambda arg: arg - input_desc.origin,
        lambda d_in: d_in,
    )


then_center = to_then(_make_center)
