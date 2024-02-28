'''
# init
>>> from opendp.mod import enable_features
>>> enable_features('contrib')

# /init

# dp-demo
>>> import opendp.prelude as dp
>>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
>>> dp_agg = base_laplace(23.4)

# /dp-demo
'''
