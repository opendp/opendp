import opendp.prelude as dp
from opendp.extras.numpy import then_np_clamp
import pytest

np = pytest.importorskip("numpy")


def test_clamp():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.random.normal(size=(100_000, 10))
    assert trans.output_domain.member(trans(data))


def test_clamp_nan_inf():
    space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    data = np.array([[np.inf, 0.0], [np.inf, np.inf], [1.0, 1.0], [np.nan, 0.0]])
    assert trans.output_domain.member(trans(data))


@pytest.mark.parametrize("num_columns", [None, 2])
def test_clamp_zero_length(num_columns):
    domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = domain, dp.symmetric_distance()
    trans = space >> then_np_clamp(norm=1.0, p=2)
    empty = np.zeros((0, 2))
    assert domain.member(empty)
    assert trans(empty).shape == (0, 2)


def test_clamp_drops_cardinalities():
    # clamping is not injective, so the output domain must not claim the
    # input's cardinalities
    domain = dp.numpy.array2_domain(num_columns=2, T=float, cardinalities=[3, 3])
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(norm=1.0, p=2)
    assert trans.output_domain.descriptor.cardinalities is None


def test_clamp_rejects_length_one_origin():
    # a length-1 origin silently broadcasts over wider data
    with pytest.raises(ValueError, match="origin must have more than one value"):
        then_np_clamp(norm=1.0, p=2, origin=np.array([1.0]))


def test_clamp_origin():
    domain = dp.numpy.array2_domain(num_columns=2, T=float)
    trans = (domain, dp.symmetric_distance()) >> then_np_clamp(
        norm=1.0, p=2, origin=np.array([0.5, -0.5])
    )
    data = np.random.normal(size=(1000, 2))
    assert trans.output_domain.member(trans(data))
