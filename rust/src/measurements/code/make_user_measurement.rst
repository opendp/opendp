>>> dp.enable_features("contrib")
>>> def const_function(_arg):
...     return 42
>>> def privacy_map(_d_in):
...     return 0.
>>> user_measurement = dp.m.make_user_measurement(
...     dp.atom_domain(T=int),
...     dp.absolute_distance(int),
...     dp.max_divergence(float),
...     const_function,
...     privacy_map,
...     TO=dp.RuntimeType.infer(42),
... )
>>> print('42?', user_measurement(0))
42? 42

