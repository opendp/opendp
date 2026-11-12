"""Tests for Gem+ on mbi 2.

Covers gem_plus and GemPlusModel's interface, the input checks, and end-to-end
releases with Gem+ as the estimator inside OpenDP's AIM and MST.
"""

from collections import Counter
from itertools import pairwise

import pytest

pytest.importorskip("mbi")
jax = pytest.importorskip("jax")

import jax.numpy as jnp  # type: ignore[import-untyped,import-not-found]
import numpy as np
import optax  # type: ignore[import-untyped,import-not-found]
from mbi import Domain, LinearMeasurement  # type: ignore[import-untyped,import-not-found]
from mbi.marginal_loss import from_linear_measurements  # type: ignore[import-untyped,import-not-found]

from opendp.extras.mbi._gem import GemPlusEstimator, GemPlusModel, gem_plus

jax.config.update("jax_enable_x64", True)  # same precision gem_plus uses


TOTAL = 100.0
BATCH = 50
DIM = 8
HIDDEN = (16, 16)
FAST = dict(optimizer=optax.adam(1e-3), dim=DIM, batch=BATCH, hidden=HIDDEN, steps=50)


@pytest.fixture
def domain():
    return Domain.fromdict({"age": 3, "income": 2, "region": 4})


@pytest.fixture
def model(domain):
    return GemPlusModel.random(domain, TOTAL, dim=DIM, batch=BATCH, hidden=HIDDEN)


def _measurements(domain):
    rng = np.random.default_rng(0)
    cliques = [("age",), ("age", "income"), ("income", "region")]
    return [
        LinearMeasurement(jnp.asarray(rng.random(domain.size(cl)) * 10), cl, stddev=1.0)
        for cl in cliques
    ]


def test_returns_gem_plus_model(domain):
    # No optimizer or hidden: covers the defaults (Adam 1e-4, two blocks of 2 * dim).
    fitted = gem_plus(domain, _measurements(domain), dim=DIM, batch=BATCH, steps=50)
    assert isinstance(fitted, GemPlusModel)
    assert fitted.domain == domain
    assert fitted.project("age").values.shape == (3,)
    assert fitted.num_components() == BATCH


def test_supports(model):
    assert model.supports(("age", "income"))
    assert model.supports("region")
    assert not model.supports(("age", "zip"))


def test_batch_of_one_rejected(domain):
    with pytest.raises(ValueError, match="at least 2"):
        GemPlusModel.random(domain, TOTAL, dim=DIM, batch=1, hidden=HIDDEN)


def test_steps_must_be_multiple_of_50(domain):
    with pytest.raises(ValueError, match="multiple of 50"):
        gem_plus(domain, _measurements(domain), **{**FAST, "steps": 120})

def test_marginals_consistent_and_scaled(model):
    ab = model.project(("age", "income")).values
    assert np.isclose(ab.sum(), TOTAL)
    assert np.allclose(ab.sum(axis=1), model.project("age").values)

def test_rejects_marginal_loss_fn(domain):
    loss = from_linear_measurements(_measurements(domain), domain)
    with pytest.raises(TypeError, match="MarginalLossFn"):
        gem_plus(domain, loss, **FAST)


def test_warm_start_wrong_type(domain):
    with pytest.raises(TypeError, match="GemPlusModel"):
        gem_plus(domain, _measurements(domain), warm_start=object(), **FAST)


def test_warm_start_wrong_domain(domain):
    other = GemPlusModel.random(
        Domain.fromdict({"age": 3, "income": 2}),
        TOTAL,
        dim=DIM,
        batch=BATCH,
        hidden=HIDDEN,
    )
    with pytest.raises(ValueError, match="different domain"):
        gem_plus(domain, _measurements(domain), warm_start=other, **FAST)


def test_constraints_not_supported(domain):
    estimator = GemPlusEstimator(
        optimizer=optax.adam(1e-3), dim=DIM, batch=BATCH, hidden=HIDDEN
    )
    with pytest.raises(NotImplementedError):
        estimator.estimate(
            domain, _measurements(domain), constraints=[object()], iters=50
        )


def _release_with_gem_plus(make_algorithm):
    """Releases a contingency table with gem_plus as the estimator and checks
    every point where OpenDP touches the model: the table's first fit
    (potentials=None), warm starts after that, one z for the whole run, and
    synthesize(). Returns the keyword arguments and measured cliques of each
    estimator call."""
    pl = pytest.importorskip("polars")
    import opendp.prelude as dp

    dp.enable_features("contrib")

    rng = np.random.default_rng(0)
    n = 500
    a = rng.integers(0, 3, n)
    b = (a + rng.integers(0, 2, n)) % 2
    c = rng.integers(0, 4, n)
    data = pl.LazyFrame({"a": a, "b": b, "c": c})
    keys = {"a": [0, 1, 2], "b": [0, 1], "c": [0, 1, 2, 3]}

    calls, fitted, measured = [], [], []

    def estimator(domain, loss_fn, **kwargs):
        calls.append(kwargs)
        measured.append([m.clique for m in loss_fn])
        model = gem_plus(domain, loss_fn, **kwargs, **FAST)
        fitted.append(model)
        return model

    context = dp.Context.compositor(
        data=data,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=1.0),
    )
    table = (
        context.query(rho=1.0)
        .contingency_table(keys=keys, algorithm=make_algorithm(dp, estimator))
        .release()
    )

    assert calls[0] == {"potentials": None}  # the table's first fit
    for kwargs in calls[1:]:
        assert isinstance(kwargs.get("warm_start"), GemPlusModel)
    for model in fitted:
        assert jnp.array_equal(model._z, fitted[0]._z)

    assert isinstance(table.model, GemPlusModel)
    synth = table.synthesize()
    assert set(synth.columns) == {"a", "b", "c"}
    assert synth.height > 0
    return calls, measured


def test_aim_end_to_end_with_gem_plus():
    calls, _ = _release_with_gem_plus(
        lambda dp, est: dp.mbi.AIM(estimator=est, rounds=3)
    )
    assert len(calls) >= 2, "AIM ran no rounds"


def test_aim_with_gem_plus_does_not_reselect():
    # More rounds than the 3 columns have marginals, so without the filter
    # AIM would select some marginal twice.
    _, measured = _release_with_gem_plus(
        lambda dp, est: dp.mbi.AIM(estimator=est, rounds=20, gem_filter=True)
    )
    # each round adds its selected marginal; no two rounds select the same one
    added = []
    for before, after in pairwise(measured):
        added += list((Counter(after) - Counter(before)).elements())
    assert len(added) == len(set(added))


def test_mst_end_to_end_with_gem_plus():
    calls, _ = _release_with_gem_plus(lambda dp, est: dp.mbi.MST(estimator=est))
    assert len(calls) == 2  # the table's first fit, then one fit on the tree


def test_closure_averages_estimates_of_the_same_clique(domain):
    from opendp.extras.mbi._gem import _marginal_closure

    rng = np.random.default_rng(0)

    def measure(clique, stddev):
        values = jnp.asarray(rng.random(domain.size(clique)) * 10)
        return LinearMeasurement(values, clique, stddev=stddev)

    # (age, income) measured directly in the opposite column order,
    # and derived from (age, income, region)
    direct = measure(("income", "age"), 1.0)
    parent = measure(("age", "income", "region"), 2.0)
    total = LinearMeasurement(jnp.array([500.0]), (), stddev=1.0)
    closed = _marginal_closure(domain, [direct, parent, total])

    assert total in closed  # returned unchanged
    # every clique appears once
    assert len({frozenset(m.clique) for m in closed}) == len(closed)

    (found,) = [m for m in closed if set(m.clique) == {"age", "income"}]
    assert found.clique == ("age", "income")

    from_direct = jnp.reshape(direct.noisy_measurement, (2, 3)).T.flatten()
    from_parent = jnp.reshape(parent.noisy_measurement, (3, 2, 4)).sum(axis=2).flatten()
    w_direct, w_parent = 1 / 1.0, 1 / (2.0 * 2)  # parent sums 4 region cells
    expected = (w_direct * from_direct + w_parent * from_parent) / (w_direct + w_parent)
    assert jnp.allclose(found.noisy_measurement, expected)

