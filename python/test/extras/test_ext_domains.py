from opendp.extras.numpy import _sscp_domain
import opendp.prelude as dp
import pytest


def test_array2_domain():
    np = pytest.importorskip("numpy")
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


def test_array2_domain_member():
    np = pytest.importorskip("numpy")
    # missing norm
    domain = dp.numpy.array2_domain(norm=1, p=1, nan=False, size=2, num_columns=2, T=float)

    with pytest.warns(match="must be a numpy ndarray"):
        domain.member([1, 2, 3])
    with pytest.warns(match="must have data of type f64, got i64"):
        domain.member(np.array([[1, 2, 3]]))
    with pytest.warns(match="must be a 2-dimensional array"):
        domain.member(np.array([1.0, 2.0, 3.0]))
    with pytest.warns(match="must have 2 columns"):
        domain.member(np.array([[1.0, 2.0, 3.0]]))
    with pytest.warns(match="must not contain NaN values"):
        domain.member(np.array([[float("NaN"), 2.0]]))
    with pytest.warns(match="must have row norm at most 1"):
        domain.member(np.array([[1.0, 2.0]]))
    with pytest.warns(match="must have exactly 2 rows"):
        domain.member(np.array([[1.0, 0.0]]))
    assert domain.member(np.array([[1.0, 0.0], [1.0, 0.0]]))


def test_sscp_domain():
    np = pytest.importorskip("numpy")
    
    with pytest.raises(ValueError):
        _sscp_domain(T=bool)

    domain = _sscp_domain(num_features=2, T=float)

    with pytest.warns(match="must be a numpy ndarray"):
        domain.member(False)
    with pytest.warns(match="must have data of type"):
        domain.member(np.array([[1, 2, 3]]))
    with pytest.warns(match="must be a square array"):
        domain.member(np.array([[1.0, 2.0, 3.0]]))
    with pytest.warns(match="must have 2 features"):
        domain.member(np.random.normal(size=(3, 3)))
    with pytest.warns(match="must have finite values"):
        domain.member(np.array([[float("NaN"), 2.0], [2.0, 1.0]]))
    with pytest.warns(match="must be symmetric"):
        domain.member(np.array([[1.0, 2.0], [1.0, 2.0]]))
    with pytest.warns(match="must be positive semi-definite"):
        domain.member(np.array([[1.0, 2.0], [2.0, 1.0]]))

    assert domain.member(np.array([[2.0, 1.0], [1.0, 2.0]]))
