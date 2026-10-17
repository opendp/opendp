import opendp.prelude as dp
from opendp.extras.numpy import then_np_clip
import pytest
from ..helpers import optional_dependency


def test_clip():
    with optional_dependency("numpy"):
        space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clip(norm=1.0, p=2)
    np = pytest.importorskip("numpy")
    data = np.random.normal(size=(100_000, 10))
    assert trans.output_domain.member(trans(data))


def test_clip_nan_inf():
    with optional_dependency("numpy"):
        space = dp.numpy.array2_domain(T=float), dp.symmetric_distance()
    trans = space >> then_np_clip(norm=1.0, p=2)
    np = pytest.importorskip("numpy")
    data = np.array([[np.inf, 0.0], [np.inf, np.inf], [1.0, 1.0], [np.nan, 0.0]])
    assert trans.output_domain.member(trans(data))


@pytest.mark.parametrize("dtype,T", [("int32", dp.i32), ("float32", dp.f32), ("float64", dp.f64)])
def test_clamp_output_dtype_and_cardinalities(dtype, T):
    np = pytest.importorskip("numpy")
    data = np.array([[1, 1], [1, 0]], dtype=dtype)
    domain = dp.numpy.array2_domain(T=T, size=2, num_columns=2, cardinalities=[1, 2])
    trans = (domain, dp.symmetric_distance()) >> then_np_clip(norm=1.0, p=2)
    result = trans(data)
    assert result.dtype == np.float64
    assert len(np.unique(result[:, 0])) == 2
    assert trans.output_domain.descriptor.cardinalities is None
    assert trans.output_domain.member(result)


def test_clamp_empty_data():
    np = pytest.importorskip("numpy")
    domain = dp.numpy.array2_domain(T=float, size=0, num_columns=2)
    trans = (domain, dp.symmetric_distance()) >> then_np_clip(norm=1.0, p=2)
    result = trans(np.zeros((0, 2)))
    assert result.shape == (0, 2)
    assert trans.output_domain.member(result)


def test_clip_projects_rows_and_updates_domain():
    np = pytest.importorskip("numpy")
    domain = dp.numpy.array2_domain(
        size=1,
        num_columns=2,
        nan=False,
        T=float,
    )
    transformation = (
        domain,
        dp.symmetric_distance(),
    ) >> then_np_clip(norm=1.0, p=2)

    assert np.allclose(transformation(np.array([[3.0, 4.0]])), [[0.6, 0.8]])
    descriptor = transformation.output_domain.descriptor
    assert descriptor.norm == 1.0
    assert descriptor.p == 2
    assert descriptor.nan is False
