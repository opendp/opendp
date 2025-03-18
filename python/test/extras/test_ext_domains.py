from opendp.extras.numpy import _sscp_domain
import opendp.prelude as dp
import pytest



def test_array2_domain():
    np = pytest.importorskip('numpy')
    # missing norm
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(p=2, T=float)
    # origin is wrong type
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(norm=1, p=2, origin="a", T=float)
    # scalar origin must be at zero
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(norm=1, p=2, origin=2, T=float)
    # origin must be consistent with num_columns
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(
            norm=1, p=2, origin=np.array([1, 2]), num_columns=3, T=float
        )
    # origin array dtype must be numeric
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(norm=1, p=2, origin=np.array([True, False]))
    
    with pytest.raises(ValueError):
        dp.numpy.array2_domain(norm=1, p=2, T="AnyTransformation")

    # origin defaults to zero
    assert dp.numpy.array2_domain(norm=1, p=2, T=float).descriptor.origin == 0
    # when num columns known, origin defaults to zero vector
    domain = dp.numpy.array2_domain(norm=1, p=2, num_columns=4)
    assert np.array_equal(domain.descriptor.origin, np.zeros(4))
    assert domain.member(np.array([[1.0, 0.0, 0.0, 0.0]]))

    domain = dp.numpy.array2_domain(norm=1, p=2, origin=np.array([1, 2]), T=float)
    assert domain.descriptor.num_columns == 2
    assert domain.descriptor.origin.dtype.kind == "f"

    assert dp.numpy.array2_domain(T=bool).member(np.array([[True, False]]))


def test_sscp_domain():
    np = pytest.importorskip('numpy')

    domain = _sscp_domain(num_features=4, T=float)
    assert domain.member(np.random.normal(size=(4, 4)))

    domain = _sscp_domain(num_features=4, T=dp.f32)
    assert domain.member(np.random.normal(size=(4, 4)).astype(np.float32))

    with pytest.warns(UserWarning, match=r'does not belong to carrier type'):
        domain.member(np.random.normal(size=(4, 4)))

    with pytest.raises(ValueError):
        _sscp_domain(T=bool)
