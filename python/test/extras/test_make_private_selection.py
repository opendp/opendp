import math

import opendp.prelude as dp
import pytest

dp.enable_features("contrib")


def make_laplace_scorer():
    np = pytest.importorskip("numpy")
    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    return space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: (float(np.random.laplace(loc=x, scale=1.0)), "candidate"),
        lambda d_in: d_in,
        TO="(f64, ExtrinsicObject)",
    )


def make_gaussian_scorer():
    np = pytest.importorskip("numpy")
    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    return space >> dp.m.then_user_measurement(
        dp.renyi_divergence(),
        lambda x: (float(np.random.normal(loc=x, scale=1.0)), "candidate"),
        lambda d_in: lambda alpha: d_in**2 * alpha / 2.0,
        TO="(f64, ExtrinsicObject)",
    )


def test_private_selection_threshold_composition():
    np = pytest.importorskip("numpy")
    bounds = 0.0, 100.0
    epsilon = 1.0
    threshold = 23.0
    space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()

    m_count = space >> dp.t.then_count() >> dp.m.then_laplace(scale=2 / epsilon)
    m_sum = (
        space
        >> dp.t.then_impute_constant(0.0)
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=2 * max(abs(bounds[0]), bounds[1]) / epsilon)
    )
    m_scored_candidate = dp.c.make_composition([m_count, m_sum])

    m_select = dp.c.make_select_above_threshold(
        m_scored_candidate, mean=100.0, threshold=threshold
    )
    data = np.random.default_rng(seed=42).normal(10, 5, 20)
    release = m_select(data)
    assert release is None or release[0] >= threshold
    assert m_select.map(1) == 2 * m_scored_candidate.map(1)

    # The original API remains callable with its original argument names.
    legacy = dp.c.make_select_private_candidate(
        m_scored_candidate, stop_probability=0.5, threshold=threshold
    )
    assert legacy.map(1) == 2 * m_scored_candidate.map(1)


def test_private_selection_best_candidate():
    m_plugin = make_laplace_scorer()
    score, candidate = dp.c.make_select_best_candidate(m_plugin, mean=2.0)(20.0)
    assert isinstance(score, float)
    assert candidate == "candidate"
    assert dp.c.make_select_best_candidate(m_plugin, mean=2.0).map(1.0) == 3.0


def test_private_selection_no_answer():
    m_plugin = make_laplace_scorer()
    m_select = dp.c.make_select_above_threshold(
        m_plugin, mean=100.0, threshold=10_000.0
    )
    assert m_select(20.0) is None
    assert m_select.map(1.0) == 2.0


def test_private_selection_renyi_threshold():
    m_plugin = make_gaussian_scorer()
    gamma = 0.01
    m_select = dp.c.make_select_above_threshold(
        m_plugin, mean=1.0 / gamma, threshold=23.0
    )
    curve = m_select.map(1.0)
    assert math.isinf(curve(2.0))
    expected = 2.0 + (2.0 / 3.0) * 1.5 + 2.0 * math.log(1.0 / gamma) / 3.0
    assert abs(curve(4.0) - expected) < 1e-12


def test_private_selection_renyi_best_candidate():
    m_plugin = make_gaussian_scorer()
    mean = 2.0
    m_select = dp.c.make_select_best_candidate(m_plugin, mean=mean)
    curve = m_select.map(1.0)
    expected = 2.0 + 2.0 * (1.0 - 1.0 / 4.0) * 2.0
    expected += 2.0 * math.log(mean) / 4.0 + math.log(mean) / 3.0
    assert abs(curve(4.0) - expected) < 1e-12


def test_private_selection_negative_binomial_and_poisson():
    m_plugin = make_gaussian_scorer()

    m_nb = dp.c.make_select_best_candidate(
        m_plugin, mean=2.5, distribution="negative_binomial", eta=0.5
    )
    assert math.isfinite(m_nb.map(1.0)(4.0))

    # A Poisson mean below one must not produce negative Rényi divergence.
    m_poisson = dp.c.make_select_best_candidate(
        m_plugin, mean=0.5, distribution="poisson"
    )
    assert m_poisson.map(1.0)(4.0) >= 0.0


def test_private_selection_mean_one():
    m_plugin = make_laplace_scorer()
    assert (
        dp.c.make_select_above_threshold(m_plugin, mean=1.0, threshold=0.0).map(1.0)
        == 1.0
    )

    for distribution in ("geometric", "logarithmic"):
        selected = dp.c.make_select_best_candidate(
            m_plugin, mean=1.0, distribution=distribution
        )
        assert selected.map(1.0) == 1.0

    selected = dp.c.make_select_best_candidate(
        m_plugin, mean=1.0, distribution="negative_binomial", eta=0.5
    )
    assert selected.map(1.0) == 1.0


def test_private_selection_rejects_unsupported_combinations():
    m_pure = make_laplace_scorer()
    with pytest.raises(Exception):
        dp.c.make_select_best_candidate(m_pure, mean=2.0, distribution="poisson")
    with pytest.raises(Exception):
        dp.c.make_select_best_candidate(
            m_pure, mean=2.0, distribution="negative_binomial", eta=-1.0
        )
    with pytest.raises(Exception):
        dp.c.make_select_above_threshold(m_pure, mean=1.0, threshold=1.0)

    m_rdp = make_gaussian_scorer()
    with pytest.raises(Exception):
        dp.c.make_select_best_candidate(m_rdp, mean=-1.0, distribution="poisson")
