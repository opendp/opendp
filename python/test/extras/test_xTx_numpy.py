from typing import Any

import opendp.prelude as dp
import pytest
from opendp.extras.numpy._make_np_sscp import make_np_sscp, then_np_sscp

np = pytest.importorskip("numpy")


def test_np_sscp_sym():
    space = (
        dp.numpy.array2_domain(num_columns=4, T=float, nan=False, norm=10.0, p=2),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp(dp.nuclear_distance())
    data = np.random.normal(size=(1000, 4))
    assert np.array_equal(trans(data), data.T @ data)
    assert trans.map(1) == 100.0


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

    if constructor == "eigendecomposition":
        pytest.importorskip("sklearn")
    with pytest.raises(ValueError, match="origin must be zero"):
        if constructor == "sscp":
            make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())
        else:
            (domain, dp.symmetric_distance()) >> (
                dp.sklearn.decomposition.then_private_np_eigendecomposition(
                    eigvals_epsilon=1.0, eigvecs_epsilons=[1.0]
                )
            )


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
@pytest.mark.parametrize(
    "origin_kind", ["default", "scalar_zero", "zero_vector", "signed_zero"]
)
def test_zero_origin_preserves_sscp(T, dtype, origin_kind):
    kwargs: dict[str, Any] = {"norm": 1.0, "p": 2}
    if origin_kind == "scalar_zero":
        kwargs["origin"] = 0.0
    elif origin_kind == "zero_vector":
        kwargs["origin"] = np.zeros(2, dtype=dtype)
    elif origin_kind == "signed_zero":
        kwargs["origin"] = np.array([-0.0, 0.0], dtype=dtype)
    domain = dp.numpy.array2_domain(num_columns=2, size=2, nan=False, T=T, **kwargs)
    trans = make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())
    data = np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)

    assert domain.member(data)
    actual = trans(data)
    assert np.array_equal(actual, data.T @ data)
    assert actual.dtype == data.dtype
    assert trans.output_domain.member(actual)
    assert trans.map(1) == 0.0


@pytest.mark.parametrize("norm", [None, float("nan"), float("inf")])
def test_np_sscp_requires_finite_norm(norm):
    kwargs = {} if norm is None else {"norm": norm, "p": 2}
    domain = dp.numpy.array2_domain(num_columns=2, T=float, nan=False, **kwargs)
    with pytest.raises(ValueError, match="finite row norm"):
        make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())


def test_np_sscp_nuclear_sensitivity():
    B = 2.0
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=float, norm=B, p=2, nan=False
    )
    trans = make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())

    assert trans.output_metric == dp.nuclear_distance()
    assert trans.map(1) == 0.0
    assert trans.map(2) == pytest.approx(2 * B**2)

    x = np.array([B, 0.0])
    y = np.array([0.0, B])
    delta = np.outer(x, x) - np.outer(y, y)
    nuclear_distance = np.linalg.svd(delta, compute_uv=False).sum()
    assert nuclear_distance == trans.map(2)
    assert trans.map(3) == trans.map(2)


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
def test_nuclear_sscp_does_not_construct_native_lp_metrics(T, dtype, monkeypatch):
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=T, norm=1.0, p=2, nan=False
    )
    nuclear_metric = dp.nuclear_distance()

    def unexpected(*args, **kwargs):
        raise AssertionError("Nuclear SSCP must not construct unrelated Lp metrics")

    monkeypatch.setattr(dp, "l1_distance", unexpected)
    monkeypatch.setattr(dp, "l2_distance", unexpected)
    trans = make_np_sscp(domain, dp.symmetric_distance(), nuclear_metric)
    data = np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)
    result = trans(data)
    np.testing.assert_array_equal(result, data.T @ data)
    assert result.dtype == dtype
    assert trans.output_domain.member(result)
    assert trans.output_metric == nuclear_metric
    assert trans.map(1) == 0.0
    assert trans.map(2) == pytest.approx(2.0)


@pytest.mark.parametrize(
    "norm, error",
    [
        (1e-200, "row norm squared underflows"),
        (1e200, "row norm squared overflows"),
    ],
)
def test_nuclear_sscp_rejects_unrepresentable_sensitivity(norm, error):
    domain = dp.numpy.array2_domain(num_columns=2, T=float, norm=norm, p=2, nan=False)
    with pytest.raises(ValueError, match=error):
        make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())


def test_nuclear_sscp_scalar_substitution():
    domain = dp.numpy.array2_domain(
        num_columns=1, size=2, T=float, norm=2.0, p=2, nan=False
    )
    trans = make_np_sscp(domain, dp.symmetric_distance(), dp.nuclear_distance())
    assert trans.map(1) == 0.0
    assert trans.map(2) == 4.0
    assert trans.map(4) == 8.0
