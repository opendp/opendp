import math

import opendp.prelude as dp
import pytest

dp.enable_features("contrib", "private-selection-v2")


def test_private_selection_threshold_composition():
    bounds = 0.0, 100.0
    range_ = max(abs(bounds[0]), bounds[1])
    epsilon = 1.0
    threshold = 23

    space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()

    m_count = space >> dp.t.then_count() >> dp.m.then_laplace(scale=2 / epsilon)
    m_sum = (
        space
        >> dp.t.then_impute_constant(0.0)
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=2 * range_ / epsilon)
    )

    m_scored_candidate = dp.c.make_composition([m_count, m_sum])
    m_private_selection = dp.c.make_select_private_candidate(
        m_scored_candidate,
        mean=100.0,
        threshold=threshold,
    )

    np = pytest.importorskip("numpy")
    data = np.random.default_rng(seed=42).normal(10, 5, 20)

    release = m_private_selection(data)

    assert release is None or release[0] >= threshold
    assert m_private_selection.map(1) == 2 * m_scored_candidate.map(1)
    if release is not None:
        assert isinstance(release[1], float)


def test_private_selection_without_threshold():
    np = pytest.importorskip("numpy")

    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)

    m_plugin = space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: (np.random.normal(loc=x), x),
        lambda d_in: d_in,
        TO="(f64, ExtrinsicObject)"
    )

    score, candidate = dp.c.make_select_private_candidate(m_plugin, mean=2.0)(20)

    assert isinstance(score, float)
    assert isinstance(candidate, float)
    assert dp.c.make_select_private_candidate(m_plugin, mean=2.0).map(1) == 3 * m_plugin.map(1)


def test_private_selection_no_answer():
    np = pytest.importorskip("numpy")
    threshold = 10_000

    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)

    m_plugin = space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: (np.random.normal(loc=x), x),
        lambda d_in: d_in,
        TO="(f64, ExtrinsicObject)"
    )

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin, mean=1000.0, threshold=threshold
    )

    assert m_private_selection(20) is None
    assert m_private_selection.map(1) == 2 * m_plugin.map(1)


def test_private_selection_renyi_threshold():
    threshold = 23
    gamma = 0.01

    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    m_plugin = space >> dp.m.then_user_measurement(
        dp.renyi_divergence(),
        lambda x: (x, x),
        lambda d_in: (lambda alpha: d_in * alpha / 2.0),
        TO="(f64, ExtrinsicObject)"
    )

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin,
        mean=1.0 / gamma,
        threshold=threshold,
    )

    curve = m_private_selection.map(1.0)
    assert math.isinf(curve(2.0))
    expected = 2.0 + (2.0 / 3.0) * 1.5 + 2.0 * math.log(1.0 / gamma) / 3.0
    assert abs(curve(4.0) - expected) < 1e-12


def test_private_selection_renyi_without_threshold():
    gamma = 0.5
    mean = 1.0 / gamma

    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    m_plugin = space >> dp.m.then_user_measurement(
        dp.renyi_divergence(),
        lambda x: (x, x),
        lambda d_in: (lambda alpha: d_in * alpha / 2.0),
        TO="(f64, ExtrinsicObject)"
    )

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin,
        mean=mean,
    )

    curve = m_private_selection.map(1.0)
    expected = (
        2.0
        + 2.0 * (1.0 - 1.0 / 4.0) * 2.0
        + 2.0 * math.log(1.0 / gamma) / 4.0
        + math.log(mean) / 3.0
    )
    assert abs(curve(4.0) - expected) < 1e-12


def test_private_selection_rejects_unsupported_combinations():
    np = pytest.importorskip("numpy")
    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)

    m_pure = space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: (np.random.normal(loc=x), x),
        lambda d_in: d_in,
        TO="(f64, ExtrinsicObject)"
    )
    with pytest.raises(Exception):
        dp.c.make_select_private_candidate(m_pure, mean=2.0, threshold=1.0, distribution="logarithmic")
    with pytest.raises(Exception):
        dp.c.make_select_private_candidate(m_pure, mean=2.0, distribution="poisson")

    m_rdp = space >> dp.m.then_user_measurement(
        dp.renyi_divergence(),
        lambda x: (x, x),
        lambda d_in: (lambda alpha: d_in * alpha / 2.0),
        TO="(f64, ExtrinsicObject)"
    )
    with pytest.raises(Exception):
        dp.c.make_select_private_candidate(m_rdp, mean=2.0, threshold=1.0, distribution="poisson")


def test_private_selection_negative_binomial_python_wrapper():
    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
    m_plugin = space >> dp.m.then_user_measurement(
        dp.renyi_divergence(),
        lambda x: (x, x),
        lambda d_in: (lambda alpha: d_in * alpha / 2.0),
        TO="(f64, ExtrinsicObject)"
    )

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin,
        mean=2.5,
        distribution="negative_binomial",
        eta=0.5,
    )

    curve = m_private_selection.map(1.0)
    assert math.isfinite(curve(4.0))
