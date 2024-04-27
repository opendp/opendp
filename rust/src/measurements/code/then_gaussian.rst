>>> dp.enable_features('contrib')
>>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> gaussian = input_space >> dp.m.then_gaussian(scale=1.0)
>>> print('100?', gaussian(100.0))
100? ...