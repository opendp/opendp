>>> dp.enable_features("contrib")
>>> def const_function(_arg):
...     return 42
>>> def privacy_map(_d_in):
...     return 0.
>>> space = dp.atom_domain(T=int), dp.absolute_distance(int)
>>> user_measurement = dp.m.make_user_measurement(
...     *space,
...     output_measure=dp.max_divergence(),
...     function=const_function,
...     privacy_map=privacy_map
... )
>>> print('42?', user_measurement(0))
42? 42

