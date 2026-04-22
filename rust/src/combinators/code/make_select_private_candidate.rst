>>> import opendp.prelude as dp
>>> dp.enable_features("contrib", "private-selection-v2")
>>> bounds = (0.0, 100.0)
>>> range_ = max(abs(bounds[0]), bounds[1])
>>> epsilon = 1.0
>>> threshold = 23
>>> input_space = (
...     dp.vector_domain(dp.atom_domain(T=float)),
...     dp.symmetric_distance(),
... )
...
>>> # For demonstration purposes, construct a measurement that releases
>>> # a tuple with a private score and candidate, using only OpenDP
>>> # transformations and measurements.
>>> m_count = (
...     input_space
...     >> dp.t.then_count()
...     >> dp.m.then_laplace(scale=2.0 / epsilon)
... )
>>> m_sum = (
...     input_space
...     >> dp.t.then_impute_constant(0.0)
...     >> dp.t.then_clamp(bounds)
...     >> dp.t.then_sum()
...     >> dp.m.then_laplace(scale=2.0 * range_ / epsilon)
... )
>>> m_scored_candidate = dp.c.make_composition([m_count, m_sum])
...
>>> m_threshold = dp.c.make_select_private_candidate(
...     m_scored_candidate, mean=100.0, threshold=threshold
... )
>>> release = m_threshold([10.0, 12.0, 15.0])
>>> assert release is None or release[0] >= threshold
>>> assert m_threshold.map(1) == 2 * m_scored_candidate.map(1)
>>> if release is not None:
...     assert isinstance(release[1], float)
...
>>> m_best = dp.c.make_select_private_candidate(
...     m_scored_candidate, mean=2.0
... )
>>> score, candidate = m_best([10.0, 12.0, 15.0])
>>> assert abs(m_best.map(1) - 3 * m_scored_candidate.map(1)) < 1e-12
>>> assert isinstance(candidate, float)
...
>>> space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> m_mock_rdp = space >> dp.m.then_user_measurement(
...     dp.renyi_divergence(),
...     lambda x: (x, "arbitrary candidate"),
...     lambda d_in: (lambda alpha: d_in * alpha / 2.0),
...     TO="(f64, ExtrinsicObject)"
... )
>>> m_best_two_trials = dp.c.make_select_private_candidate(
...     m_mock_rdp, mean=2.0, distribution="geometric"
... )
>>> curve = m_best_two_trials.map(1)
>>> assert curve(4.0) > 0.0
