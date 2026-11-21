>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")
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
>>> m_threshold = dp.c.make_select_above_threshold(
...     m_scored_candidate, mean=100.0, threshold=threshold
... )
>>> release = m_threshold([10.0, 12.0, 15.0])
>>> assert release is None or release[0] >= threshold
>>> assert m_threshold.map(1) == 2 * m_scored_candidate.map(1)
