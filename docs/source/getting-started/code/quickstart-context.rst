:orphan:

# init
>>> import opendp.prelude as dp
>>> dp.enable_features('contrib')

# /init

# demo
>>> space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> laplace_mechanism = space >> dp.m.then_laplace(scale=1.)
>>> dp_value = laplace_mechanism(123.0)

# /demo
