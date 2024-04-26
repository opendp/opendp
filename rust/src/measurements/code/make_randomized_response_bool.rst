>>> dp.enable_features("contrib")
>>> random_bool = dp.m.make_randomized_response_bool(0.99)
>>> random_bool(True)
...