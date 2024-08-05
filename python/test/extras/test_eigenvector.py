import opendp.prelude as dp
from opendp.extras.numpy import _sscp_domain
import pytest
from ..helpers import optional_dependency



def test_private_eigenvector():
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvector

    with optional_dependency('numpy'):
        space = (
            _sscp_domain(num_features=4, norm=1.0, p=2, T=float),
            dp.symmetric_distance(),
        )
    with optional_dependency('randomgen'):
        meas = space >> then_private_eigenvector(unit_epsilon=100_000.0)
    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(4, 4))
    data += data.T
    noisy = meas(data)
    exact = np.linalg.eigh(data)[1][:, -1]

    # normalize sign
    noisy = np.copysign(noisy, exact)

    assert np.allclose(noisy, exact, atol=0.3)
    assert meas.map(2) == 100_000.0


def test_eigenvector_integration():
    from opendp.extras.numpy import then_np_clamp
    from opendp.extras.numpy._make_np_sscp import then_np_sscp
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvector

    num_columns = 4
    with optional_dependency('numpy'):
        domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = (
        domain,
        dp.symmetric_distance(),
    )
    with optional_dependency('randomgen'):
        meas = (
            space
            >> then_np_clamp(norm=1.0, p=2)
            >> then_np_sscp(dp.symmetric_distance())
            >> then_private_eigenvector(1.0)
        )

    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, num_columns))
    print(meas(data))


def test_eigenvectors():
    from opendp.extras.numpy import then_np_clamp
    from opendp.extras.numpy._make_np_sscp import then_np_sscp
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvectors

    num_columns = 4
    with optional_dependency('numpy'):
        domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = (
        domain,
        dp.symmetric_distance(),
    )
    sp_sscp = (
        space
        >> then_np_clamp(norm=4.0, p=2)
        >> then_np_sscp(dp.symmetric_distance())
    )
    with optional_dependency('scipy.linalg'):
        meas = sp_sscp >> then_private_eigenvectors([1.0] * 3)

    np = pytest.importorskip('numpy')
    data = np.random.normal(size=(1000, num_columns))
    with optional_dependency('randomgen'):
        print(meas(data))
