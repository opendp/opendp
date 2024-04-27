>>> dp.enable_features("contrib")
>>> input_space = dp.vector_domain(dp.atom_domain(T=int)), dp.linf_distance(T=int)
>>> select_index = dp.m.make_report_noisy_max_gumbel(*input_space, scale=1.0, optimize='Max')
>>> print('2?', select_index([1, 2, 3, 2, 1]))
2? ...