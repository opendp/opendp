from __future__ import annotations
from typing import NamedTuple, List, Optional

from opendp.extrinsics.make_np_clamp import then_np_clamp
from opendp.extrinsics._utilities import register_measurement, to_then
from opendp.extrinsics._make_np_mean import make_private_np_mean
from opendp.extrinsics._make_np_eigendecomposition import (
    then_private_np_eigendecomposition,
)
from opendp.mod import Measurement


class PCABudget(NamedTuple):
    eigvals: float
    eigvecs: List[float]
    mean: Optional[float]


def make_private_np_pca(
    input_domain,
    input_metric,
    epsilon: float | PCABudget,
    num_components=None,
    norm=None,
) -> Measurement:
    """
    :param input_domain: instance of `np_array2_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param unit_epsilon: ε-expenditure per changed record in the input data
    :param num_components: optional, number of eigenvectors to release. defaults to num_columns from input_domain
    :param norm: clamp each row to this norm bound

    :returns a Measurement that computes a tuple of (mean, eigvals, eigvecs)
    """
    import opendp.prelude as dp
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    class PCAResult(NamedTuple):
        mean: np.ndarray
        S: np.ndarray
        Vt: np.ndarray

    if input_domain.size is None:
        raise ValueError("input_domain's size must be known")

    if input_domain.num_columns is None:
        raise ValueError("input_domain's num_columns must be known")

    if input_domain.p not in {None, 2}:
        raise ValueError("input_domain's norm must be an L2 norm")

    if input_domain.num_columns < 1:
        raise ValueError("input_domain's num_columns must be >= 1")
    
    if isinstance(epsilon, float):
        num_eigvec_releases = min(num_components, input_domain.num_features - 1)
        epsilon = split_pca_epsilon_evenly(
            epsilon, num_eigvec_releases, estimate_mean=True
        )
    
    if not isinstance(epsilon, PCABudget):
        raise ValueError("epsilon must be a float or instance of PCABudget")
    
    eigvals_epsilon, eigvecs_epsilons, mean_epsilon = epsilon

    def eig_to_SVt(decomp):
        eigvals, eigvecs = decomp
        return np.sqrt(eigvals)[::-1], eigvecs[:, ::-1].T
    
    def make_eigdecomp(origin):
        return (
            (input_domain, input_metric)
            >> then_np_clamp(norm, p=2, origin=origin)
            >> then_center()
            >> then_private_np_eigendecomposition(eigvals_epsilon, eigvecs_epsilons)
            >> (lambda out: PCAResult(origin, *eig_to_SVt(out)))
        )

    if input_domain.norm is not None:
        if mean_epsilon is not None:
            raise ValueError("mean_epsilon should be zero because origin is known")
        return make_eigdecomp(input_domain.origin)

    # make releases under the assumption that d_in is 2.
    unit_d_in = 2

    epsilon_eigh = sum(eigvals_epsilon, *eigvecs_epsilons)
    compositor = dp.c.make_sequential_composition(
        input_domain,
        input_metric,
        dp.max_divergence(T=input_domain.T),
        d_in=unit_d_in,
        d_mids=[mean_epsilon, epsilon_eigh],
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
        return qbl(make_eigdecomp(origin))


    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        compositor.output_measure,
        function,
        compositor.map,
        TO="ExtrinsicObject",
    )


def _smaller(v):
    """returns the next non-negative float closer to zero"""
    import numpy as np

    if v < 0:
        raise ValueError("expected non-negative value")
    return v if v == 0 else np.nextafter(v, -1)


def split_pca_epsilon_evenly(
    unit_epsilon, num_eigvec_releases, estimate_mean=False
):
    num_queries = 3 if estimate_mean else 2
    per_query_epsilon = unit_epsilon / num_queries
    per_evec_epsilon = per_query_epsilon / num_eigvec_releases

    # use conservatively smaller budgets to prevent totals from exceeding total epsilon
    return {
        "mean_epsilon": 0 if estimate_mean else _smaller(per_query_epsilon),
        "eigval_epsilon": per_query_epsilon,
        "eigvec_epsilons": [_smaller(per_evec_epsilon)] * num_eigvec_releases,
    }


# generate then variant of the constructor
then_private_np_pca = register_measurement(make_private_np_pca)


def _make_center(input_domain, input_metric):
    import opendp.prelude as dp
    import numpy as np

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.np_array2_domain(
            **{
                **input_domain.descriptor._asdict(),
                "origin": np.zeros(input_domain.num_columns),
            }
        ),
        input_metric,
        lambda arg: arg - input_domain.origin,
        lambda d_in: d_in,
    )


then_center = to_then(_make_center)