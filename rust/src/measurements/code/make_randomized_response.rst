>>> dp.enable_features("contrib")
>>> random_string = dp.m.make_randomized_response(['a', 'b', 'c'], 0.99)
>>> print('a?', random_string('a'))
a? ...