>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")
>>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> laplace = dp.m.make_laplace(*input_space, scale=1.0)
>>> print('100?', laplace(100.0))
100? ...

Or, more readably, define the space and then chain:

>>> laplace = input_space >> dp.m.then_laplace(scale=1.0)
>>> print('100?', laplace(100.0))
100? ...