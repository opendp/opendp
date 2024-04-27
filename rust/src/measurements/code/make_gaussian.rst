>>> dp.enable_features('contrib')
>>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> gaussian = dp.m.make_gaussian(*input_space, scale=1.0)
>>> print('100?', gaussian(100.0))
100? ...