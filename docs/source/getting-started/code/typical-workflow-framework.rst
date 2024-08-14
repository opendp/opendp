:orphan:

# unit-of-privacy
>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")

>>> d_in = 1 # neighboring data set distance is at most d_in...
>>> input_metric = dp.symmetric_distance() # ...in terms of additions/removals
>>> input_domain = dp.vector_domain(dp.atom_domain(T=float))

# /unit-of-privacy


# privacy-loss
>>> d_out = 1. # output distributions have distance at most d_out (ε)...
>>> privacy_measure = dp.max_divergence(T=float) # ...in terms of pure-DP

# /privacy-loss


# public-info
>>> bounds = (0.0, 100.0)
>>> imputed_value = 50.0

# /public-info


# mediate
>>> from random import randint
>>> data = [float(randint(0, 100)) for _ in range(100)]

>>> m_sc = dp.c.make_sequential_composition(
...     input_domain=input_domain,
...     input_metric=input_metric,
...     output_measure=privacy_measure,
...     d_in=d_in,
...     d_mids=[d_out / 3] * 3,
... )

>>> # Call measurement with data to create a queryable:
>>> queryable = m_sc(data)

# /mediate


# count
>>> count_transformation = (
...     dp.t.make_count(input_domain, input_metric)
... )

>>> count_sensitivity = count_transformation.map(d_in)
>>> count_sensitivity
1

>>> count_measurement = dp.binary_search_chain(
...     lambda scale: count_transformation >> dp.m.then_laplace(scale),
...     d_in,
...     d_out / 3
... )
>>> dp_count = queryable(count_measurement)

# /count


# mean
>>> mean_transformation = (
...     dp.t.make_clamp(input_domain, input_metric, bounds) >>
...     dp.t.then_resize(size=dp_count, constant=imputed_value) >>
...     dp.t.then_mean()
... )

>>> mean_measurement = dp.binary_search_chain(
...     lambda scale: mean_transformation >> dp.m.then_laplace(scale), d_in, d_out / 3
... )

>>> dp_mean = queryable(mean_measurement)

# /mean
