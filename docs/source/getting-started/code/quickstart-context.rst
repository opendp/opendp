:orphan:

# init
>>> from opendp.mod import enable_features
>>> enable_features('contrib')

# /init

# demo
>>> import opendp.prelude as dp
>>> laplace_mechanism = dp.space_of(float) >> dp.m.then_laplace(scale=1.)
>>> dp_value = laplace_mechanism(123.0)

# /demo
