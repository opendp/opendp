>>> import numpy as np
>>> import opendp.prelude as dp
>>>
>>> dp.enable_features("contrib")
>>>
>>> # Create the randomized response mechanism
>>> m_rr = dp.m.make_randomized_response_bitvec(
...     dp.bitvector_domain(max_weight=4), dp.discrete_distance(), f=0.95
... )
>>>
>>> # compute privacy loss
>>> m_rr.map(1)
0.8006676684558611
>>>
>>> # formula is 2 * m * ln((2 - f) / f)
>>> # where m = 4 (the weight) and f = .95 (the flipping probability)
>>>
>>> # prepare a dataset to release, by encoding a bit vector as a numpy byte array
>>> data = np.packbits(
...     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
... )
>>> assert np.array_equal(data, np.array([0, 8, 12], dtype=np.uint8))
>>>
>>> # roundtrip: numpy -> bytes -> mech -> bytes -> numpy
>>> release = np.frombuffer(m_rr(data.tobytes()), dtype=np.uint8)
>>>
>>> # compare the two bit vectors:
>>> [int(bit) for bit in np.unpackbits(data)]
[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0]
>>> [int(bit) for bit in np.unpackbits(release)]
[...]