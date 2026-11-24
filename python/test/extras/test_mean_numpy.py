import opendp.prelude as dp
import pytest
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_private_np_mean():
    from opendp.extras.numpy._make_np_mean import then_private_np_mean

    space = (
        dp.numpy.array2_domain(size=1000, num_columns=4, nan=False, T=float),
        dp.symmetric_distance(),
    )
    trans = space >> then_private_np_mean(scale=0.001, norm=1.0)

    print(
        "trans(np.random.normal(size=(1000, 4)))",
        trans(np.random.normal(size=(1000, 4))),
    )
    assert 2 <= trans.map(2) < 2.000001


@pytest.mark.parametrize("norm", [None, 1.0])
def test_private_np_mean_requires_num_columns(norm):
    from opendp.extras.numpy._make_np_mean import make_private_np_mean

    domain = dp.numpy.array2_domain(norm=1.0, p=2, size=1, nan=False, T=float)
    assert domain.member(np.zeros((1, 1)))
    assert domain.member(np.zeros((1, 2)))

    with pytest.raises(ValueError, match="must specify num_columns"):
        make_private_np_mean(domain, dp.symmetric_distance(), scale=1.0, norm=norm)


@pytest.mark.parametrize("norm", [None, 1.0])
def test_private_np_mean_requires_positive_size(norm):
    from opendp.extras.numpy._make_np_mean import make_private_np_mean

    domain = dp.numpy.array2_domain(
        norm=1.0, p=2, size=0, num_columns=1, nan=False, T=float
    )
    with pytest.raises(ValueError, match="positive size"):
        make_private_np_mean(domain, dp.symmetric_distance(), scale=1.0, norm=norm)
