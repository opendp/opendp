import opendp.prelude as dp
import pytest


def test_private_selection_threshold_composition():
    bounds = 0.0, 100.0
    range_ = max(abs(bounds[0]), bounds[1])
    epsilon = 1.0
    threshold = 23

    space = dp.vector_domain(dp.atom_domain(T=float)), dp.symmetric_distance()

    # score
    m_count = space >> dp.t.then_count() >> dp.m.then_laplace(scale=2 / epsilon)

    # candidate
    m_sum = (
        space
        >> dp.t.then_clamp(bounds)
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=2 * range_ / epsilon)
    )

    m_scored_candidate = dp.c.make_basic_composition([m_count, m_sum])

    m_private_selection = dp.c.make_select_private_candidate(
        m_scored_candidate, threshold=threshold, stop_probability=0
    )

    np = pytest.importorskip("numpy")
    data = np.random.default_rng(seed=42).normal(10, 5, 20)

    score, candidate = m_private_selection(data)

    assert score >= threshold
    assert m_private_selection.map(1) == 2 * m_scored_candidate.map(1)
    assert isinstance(candidate, float)


def test_private_selection_threshold_plugin():
    np = pytest.importorskip("numpy")
    threshold = 23

    space = dp.atom_domain(T=float), dp.absolute_distance(T=float)

    m_plugin = space >> dp.m.then_user_measurement(
        dp.max_divergence(),
        lambda x: (np.random.normal(loc=x), x),
        lambda d_in: d_in,
        TO="(f64, ExtrinsicObject)"
    )

    assert m_plugin(20)[1] == 20

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin, threshold=threshold, stop_probability=0
    )

    score, candidate = m_private_selection(20)

    assert score >= threshold
    assert m_private_selection.map(1) == 2 * m_plugin.map(1)
    assert isinstance(candidate, float)


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

    assert m_plugin(20)[1] == 20

    m_private_selection = dp.c.make_select_private_candidate(
        m_plugin, threshold=threshold, stop_probability=0.001
    )

    assert m_private_selection(20) is None
    assert m_private_selection.map(1) == 2 * m_plugin.map(1)
