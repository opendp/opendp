:orphan:

# init
>>> import opendp.prelude as dp
>>> dp.enable_features('contrib')

# /init

# demo
>>> space = dp.space_of(float)
>>> laplace_mechanism = space >> dp.m.then_laplace(scale=1.)
>>> dp_value = laplace_mechanism(123.0)

# /demo
