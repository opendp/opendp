import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")

def test_randomized_response_bitvec():
    f = 1e-20
    m = 3
    m_rr = dp.m.make_randomized_response_bitvec(
        dp.bitvector_domain(max_weight=m), dp.discrete_distance(), f=f
    )

    # the postprocessor expects little endian data
    data = np.packbits(
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0],
        bitorder="little",
    )

    # roundtrip: bytes -> mech -> numpy
    release = np.frombuffer(m_rr(data), dtype=np.uint8)
    assert np.array_equal(data, release)
    # epsilon is 2 * m * ln((2 - f) / f)
    assert m_rr.map(1) == 280.4690942426452

    sums = dp.m.debias_randomized_response_bitvec([m_rr(data)] * 40, f=f)
    signs = np.packbits((np.array(sums) > 0).astype(int), bitorder="little")
    assert np.array_equal(signs, release)
