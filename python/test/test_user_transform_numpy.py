import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")


def test_numpy_user_transformation_roundtrip():
    input_is_ndarray = False

    def function(arg):
        nonlocal input_is_ndarray
        input_is_ndarray |= isinstance(arg, np.ndarray)
        return [v + 1 for v in arg]

    trans = dp.t.make_user_transformation(
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        function,
        lambda d_in: d_in,
    )

    result = dp.as_array()(trans(np.array([1, 2, 3], dtype=np.int32)))

    assert not input_is_ndarray
    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([2, 3, 4], dtype=np.int32))


def test_user_constructors():
    trans = dp.t.make_user_transformation(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain((2, 10)), 10),
        dp.symmetric_distance(),
        lambda x: [x] * 10,
        lambda d_in: d_in * 10,
    )
    assert trans(2) == [2] * 10
    assert np.array_equal(dp.as_array()(trans(2)), np.repeat(2, 10))
    assert trans.map(1) == 10

    meas = dp.m.make_user_measurement(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.max_divergence(),
        lambda x: [x] * 10,
        lambda d_in: float(d_in * 10),
        TO=dp.Vec[int],
    )

    assert meas(2) == [2] * 10
    assert np.array_equal((meas >> dp.as_array())(2), np.repeat(2, 10))
    assert meas.map(1) == 10

    assert (meas >> (lambda x: x[0]))(2) == 2

