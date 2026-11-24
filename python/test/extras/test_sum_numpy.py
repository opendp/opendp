from fractions import Fraction

import opendp.prelude as dp
import pytest

np = pytest.importorskip("numpy")

from opendp.extras.numpy._make_np_sum import make_np_sum, make_private_np_sum


@pytest.mark.parametrize("size, distance, ideal", [(None, 1, 2.0), (3, 2, 2.0)])
def test_np_sum(size, distance, ideal):
    # Fixed size cancels the origin; unknown size does not.
    domain = dp.numpy.array2_domain(
        norm=1.0,
        p=2,
        origin=np.array([1.0, 0.0]),
        size=size,
        num_columns=2,
        nan=False,
        T=float,
    )
    trans = make_np_sum(domain, dp.symmetric_distance())
    assert ideal <= trans.map(distance) < ideal * 1.001
    data = np.array([[2.0, 0.0], [1.0, 1.0], [1.0, -1.0]])
    assert domain.member(data)
    np.testing.assert_array_equal(trans(data), [4.0, 0.0])


def test_private_np_sum():
    domain = dp.numpy.array2_domain(norm=1.0, p=2, num_columns=4, nan=False, T=float)
    meas = make_private_np_sum(
        domain, dp.symmetric_distance(), dp.zero_concentrated_divergence(), 1.0
    )
    assert 0.5 <= meas.map(1) < 0.501
    assert len(meas(np.zeros((3, 4)))) == 4


@pytest.mark.parametrize("size", [None, 1])
def test_np_sum_requires_num_columns(size):
    domain = dp.numpy.array2_domain(norm=1.0, p=2, size=size, nan=False, T=float)
    with pytest.raises(ValueError, match="must specify num_columns"):
        make_np_sum(domain, dp.symmetric_distance())


def test_float32_sum_widens_before_overflow():
    v = np.float32(1e38)
    data = np.array([v, v, v, -v, -v, -v, -v, -v], dtype=np.float32).reshape(-1, 1)
    domain = dp.numpy.array2_domain(
        norm=1e38, p=2, size=8, num_columns=1, T="f32", nan=False
    )
    trans = make_np_sum(domain, dp.symmetric_distance())
    changed = data.copy()
    changed[3, 0] = v
    with np.errstate(all="raise"):
        result, result_changed = trans(data), trans(changed)
    assert trans.output_domain.carrier_type == dp.RuntimeType.parse("Vec<f64>")
    assert result[0] == -2 * float(v)
    assert result_changed[0] == 0.0
    assert abs(result[0] - result_changed[0]) <= trans.map(2)


def test_float64_sum_overflow_rejected_at_construction():
    domain = dp.numpy.array2_domain(
        norm=1e308, p=1, size=2, num_columns=1, T="f64", nan=False
    )
    with pytest.raises((ValueError, OverflowError, dp.OpenDPException)):
        make_np_sum(domain, dp.symmetric_distance())


@pytest.mark.parametrize(
    "T,dtype,output_T", [("i8", np.int8, "i64"), ("u8", np.uint8, "u64")]
)
@pytest.mark.parametrize("size", [None, 128])
def test_small_integer_sum_has_matching_promoted_dtype(T, dtype, output_T, size):
    domain = dp.numpy.array2_domain(
        norm=1.25, p=1, size=size, max_size=128, num_columns=1, T=T, nan=False
    )
    try:
        trans = make_np_sum(domain, dp.symmetric_distance())
    except dp.OpenDPException as err:
        if "debug binary" not in str(err):
            raise
        pytest.skip("integer promotion requires a release build")
    assert trans.output_domain.carrier_type == dp.RuntimeType.parse(f"Vec<{output_T}>")
    assert trans(np.ones((128, 1), dtype=dtype))[0] == 128
    assert trans.map(1) == (2 if size is None else 0)
    assert trans.map(2) == 3
    assert trans.map(1000) == 320


@pytest.mark.parametrize("T,size", [("i64", 8), ("u64", 16)])
def test_integer_overflow_rejected_at_construction(T, size):
    domain = dp.numpy.array2_domain(
        norm=float(2**60), p=1, size=size, num_columns=1, T=T, nan=False
    )
    with pytest.raises(ValueError, match="possible integer sum overflow"):
        make_np_sum(domain, dp.symmetric_distance())


def test_unsized_sum_nonzero_origin_has_nonunderflowing_map():
    domain = dp.numpy.array2_domain(
        norm=1e-200, p=2, origin=np.array([1e-200]), T="f64", nan=False
    )
    trans = make_np_sum(domain, dp.symmetric_distance())
    data = np.array([[2e-200]])
    assert domain.member(data)
    assert trans(np.empty((0, 1)))[0] == 0.0
    assert trans(data)[0] == 2e-200
    assert 2e-200 <= trans.map(1) < 2.001e-200


def test_np_sum_permutations():
    domain = dp.numpy.array2_domain(
        norm=1e16, p=2, max_size=3, num_columns=1, T="f64", nan=False
    )
    trans = make_np_sum(domain, dp.symmetric_distance())
    a = np.array([[1e16], [-1e16], [1.0]])
    b = np.array([[1e16], [1.0], [-1e16]])
    observed = abs(trans(a)[0] - trans(b)[0])
    assert 0 < observed <= trans.map(0) < 14.0


@pytest.mark.parametrize("p", [1, 2])
def test_unsized_sum_extended_precision_origin(p):
    for center in (
        np.nextafter(np.longdouble(1), np.longdouble(2)),
        np.nextafter(np.longdouble(0), np.longdouble(1)),
    ):
        domain = dp.numpy.array2_domain(
            norm=0.0, p=p, max_size=1, origin=np.array([center]), T="f64", nan=False
        )
        trans = make_np_sum(domain, dp.symmetric_distance())
        assert Fraction(trans.map(1)) >= Fraction(*center.as_integer_ratio())


def test_np_sum_rejects_boolean_elements():
    domain = dp.numpy.array2_domain(
        norm=1.0, p=1, size=2, num_columns=1, T=bool, nan=False
    )
    with pytest.raises(ValueError, match="sum does not support input type bool"):
        make_np_sum(domain, dp.symmetric_distance())
