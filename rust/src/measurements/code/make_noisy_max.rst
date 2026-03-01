>>> dp.enable_features("contrib")
>>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
>>> select_index = dp.m.make_noisy_max(*input_space, dp.max_divergence(), scale=1.0)
>>> print('2?', select_index([1, 2, 3, 2, 1]))
2? ...

Or, more readably, define the space and then chain:

>>> select_index = input_space >> dp.m.then_noisy_max(dp.max_divergence(), scale=1.0)
>>> print('2?', select_index([1, 2, 3, 2, 1]))
2? ...