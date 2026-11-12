from typing import Any

import opendp.prelude as dp
import pytest
from opendp.extras.numpy._make_np_sscp import make_np_sscp, then_np_sscp
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_np_sscp_sym():
    space = (
        dp.numpy.array2_domain(num_columns=4, T=float, nan=False),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp()
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(1) == 1


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
@pytest.mark.parametrize("origin_kind", ["positive", "negative", "subnormal"])
@pytest.mark.parametrize("constructor", ["sscp", "eigendecomposition"])
def test_nonzero_origin_rejected(T, dtype, origin_kind, constructor):
    origin = np.zeros(2, dtype=dtype)
    if origin_kind == "positive":
        origin[0] = 10.0
    elif origin_kind == "negative":
        origin[1] = -10.0
    else:
        origin[0] = np.finfo(dtype).smallest_subnormal
    domain = dp.numpy.array2_domain(
        norm=1.0, p=2, origin=origin, num_columns=2, size=2, nan=False, T=T
    )

    with pytest.raises(ValueError, match="origin must be zero"):
        if constructor == "sscp":
            make_np_sscp(domain, dp.symmetric_distance())
        else:
            (domain, dp.symmetric_distance()) >> (
                dp.sklearn.decomposition.then_private_np_eigendecomposition(
                    eigvals_epsilon=1.0, eigvecs_epsilons=[1.0]
                )
            )


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
@pytest.mark.parametrize(
    "origin_kind", ["unbounded", "default", "scalar_zero", "zero_vector", "signed_zero"]
)
def test_zero_or_unbounded_origin_preserves_sscp(T, dtype, origin_kind):
    kwargs: dict[str, Any] = {}
    if origin_kind != "unbounded":
        kwargs.update(norm=1.0, p=2)
    if origin_kind == "scalar_zero":
        kwargs["origin"] = 0.0
    elif origin_kind == "zero_vector":
        kwargs["origin"] = np.zeros(2, dtype=dtype)
    elif origin_kind == "signed_zero":
        kwargs["origin"] = np.array([-0.0, 0.0], dtype=dtype)
    domain = dp.numpy.array2_domain(num_columns=2, size=2, nan=False, T=T, **kwargs)
    trans = make_np_sscp(domain, dp.symmetric_distance())
    data = np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)

    assert domain.member(data)
    actual = trans(data)
    assert np.array_equal(actual, data.T @ data)
    assert actual.dtype == data.dtype
    assert trans.output_domain.member(actual)
    assert trans.map(1) == 1
