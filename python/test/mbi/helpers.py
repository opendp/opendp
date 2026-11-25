"""Shared helpers for MBI tests."""

from functools import partial


def get_model(domain: dict[str, int]):
    """Construct a model for tests that only inspect metadata."""
    from mbi import CliqueVector, MarkovRandomField, Domain  # type: ignore[import-not-found]

    vector = CliqueVector(domain=Domain.fromdict(domain), cliques=[], tables={})
    return MarkovRandomField(potentials=vector, marginals=vector)


def estimate(domain, loss_fn, *, potentials=None, iters):
    import jax  # type: ignore[import-not-found]
    from mbi.estimation import MirrorDescent  # type: ignore[import-not-found]

    # Match the production estimator's FFI precision requirement.
    jax.config.update("jax_enable_x64", True)
    return MirrorDescent().estimate(
        domain,
        loss_fn,
        warm_start=potentials,
        iters=iters,
    )


fast_estimator = partial(estimate, iters=25)
effective_estimator = partial(estimate, iters=400)


def tv_distance(actual, expected):
    import numpy as np  # type: ignore[import-not-found]

    actual = np.asarray(actual, dtype=float)
    expected = np.asarray(expected, dtype=float)
    return np.abs(actual / actual.sum() - expected / expected.sum()).sum() / 2
