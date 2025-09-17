>>> dp.enable_features("contrib")
>>> input_space = dp.atom_domain(T=int), dp.absolute_distance(T=int)
>>> geometric = dp.m.make_geometric(*input_space, scale=1.0)
>>> print('100?', geometric(100))
100? ...

Or, more readably, define the space and then chain:

>>> geometric = input_space >> dp.m.then_geometric(scale=1.0)
>>> print('100?', geometric(100))
100? ...