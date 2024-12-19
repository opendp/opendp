>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")
>>> threshold = 23
>>> space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
...
>>> # For demonstration purposes-- construct a measurement that releases
>>> # a tuple with a differentially private score and value.
>>> # The tuple released must satisfy the privacy guarantee from the map.
>>> m_mock = space >> dp.m.then_user_measurement(
...     dp.max_divergence(),
...     lambda x: (np.random.laplace(loc=x), "arbitrary candidate"),
...     lambda d_in: d_in,
...     TO="(f64, ExtrinsicObject)"
... )
...
>>> m_private_selection = dp.c.make_select_private_candidate(
...     m_mock, threshold=threshold, stop_probability=0
... )
...
>>> score, candidate = m_private_selection(20)
...
>>> assert score >= threshold
>>> assert m_private_selection.map(1) == 2 * m_mock.map(1)
>>> assert isinstance(candidate, str)
