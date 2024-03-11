:orphan:

# init
>>> from opendp.mod import enable_features
>>> enable_features('contrib')

# /init

# demo
>>> import opendp.prelude as dp
>>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
>>> dp_value = base_laplace(123.0)

# /demo
