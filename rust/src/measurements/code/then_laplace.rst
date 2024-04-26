>>> dp.enable_features('contrib')
>>> input_space = dp.atom_domain(T=float), dp.absolute_distance(T=float)
>>> laplace = input_space >> dp.m.laplace(scale=1.0)
>>> laplace(100.0)
...