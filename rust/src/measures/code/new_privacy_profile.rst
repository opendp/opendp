>>> dp.enable_features("contrib", "honest-but-curious")
>>> # Prefer the vector-valued constructor over this deprecated scalar API.
>>> profile = dp.PrivacyProfile(delta=lambda eps: [1.0 if e < 0.5 else 1e-8 for e in eps])
...
>>> # epsilon is not enough, so delta saturates to one
>>> profile.delta(epsilon=0.499)
1.0
>>> # Allow for the conservative delta-to-log conversion at a plateau.
>>> import math
>>> profile.epsilon(delta=math.nextafter(profile.delta(0.5), 1.0))
0.5
>>> # insufficient delta results in infinite epsilon
>>> profile.epsilon(delta=1e-9)
inf
