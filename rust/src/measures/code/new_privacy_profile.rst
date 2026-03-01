>>> dp.enable_features("contrib", "honest-but-curious")
>>> profile = dp.new_privacy_profile(lambda eps: 1.0 if eps < 0.5 else 1e-8)
...
>>> # epsilon is not enough, so delta saturates to one
>>> profile.delta(epsilon=0.499)
1.0
>>> # invert it, find the suitable epsilon at this delta
>>> profile.epsilon(delta=1e-8)
0.5
>>> # insufficient delta results in infinite epsilon
>>> profile.epsilon(delta=1e-9)
inf
