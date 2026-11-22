from typing import Any

import opendp.prelude as dp
import pytest
from opendp.extras.numpy._make_np_sscp import make_np_sscp, then_np_sscp
from ..helpers import optional_dependency

np = pytest.importorskip("numpy")


def test_np_sscp_sym():
    space = (
        dp.numpy.array2_domain(num_columns=4, T=float, nan=False, norm=10.0, p=2),
        dp.symmetric_distance(),
    )
    trans = space >> then_np_sscp()
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
            make_np_sscp(domain, dp.symmetric_distance())
        else:
            (domain, dp.symmetric_distance()) >> (
                dp.sklearn.decomposition.then_private_np_eigendecomposition(
                    eigvals_epsilon=1.0, eigvecs_epsilons=[1.0]
                )
            )


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
@pytest.mark.parametrize("origin_kind", ["scalar_zero", "zero_vector", "signed_zero"])
def test_zero_origin_preserves_sscp(T, dtype, origin_kind):
    kwargs: dict[str, Any] = {"norm": 1.0, "p": 2}
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


@pytest.mark.parametrize("norm", [None, float("nan"), float("inf")])
def test_np_sscp_requires_finite_norm(norm):
    kwargs = {} if norm is None else {"norm": norm, "p": 2}
    domain = dp.numpy.array2_domain(num_columns=2, T=float, nan=False, **kwargs)
    with pytest.raises(ValueError, match="finite row norm"):
        make_np_sscp(domain, dp.symmetric_distance())


def test_np_sscp_nuclear_norm_sensitivity():
    B = 2.0
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=float, norm=B, p=2, nan=False
    )
    from opendp.extras.numpy import _nuclear_norm_distance

    trans = make_np_sscp(
        domain, dp.symmetric_distance(), output_metric=_nuclear_norm_distance()
    )

    assert trans.output_metric == _nuclear_norm_distance()
    assert trans.map(1) == B**2
    assert trans.map(2) == 2 * B**2

    x = np.array([B, 0.0])
    y = np.array([0.0, B])
    delta = np.outer(x, x) - np.outer(y, y)
    nuclear_distance = np.linalg.svd(delta, compute_uv=False).sum()
    assert nuclear_distance == trans.map(2)


def test_np_sscp_frobenius_substitution_sensitivity():

    norm = 2.0
    fixed = dp.numpy.array2_domain(
        num_columns=2, size=3, T=float, norm=norm, p=2, nan=False
    )
    metric = dp.symmetric_distance()
    trans = make_np_sscp(fixed, metric)
    assert trans.output_metric == dp.l2_distance(T=float)
    assert trans.map(1) == 0
    assert trans.map(2) >= np.sqrt(2) * norm**2
    assert trans.map(2) < 2 * norm**2

    x = np.array([norm, 0.0])
    y = np.array([0.0, norm])
    delta = np.outer(x, x) - np.outer(y, y)
    assert np.linalg.norm(delta, ord="fro") <= trans.map(2)

    unsized = dp.numpy.array2_domain(
        num_columns=2, T=float, norm=norm, p=2, nan=False
    )
    generic = make_np_sscp(unsized, metric)
    assert generic.map(2) == 2 * norm**2



def test_np_sscp_l1_default_and_independent_output_metrics():
    from opendp.extras.numpy import _nuclear_norm_distance

    B = 2.0
    metric = dp.symmetric_distance()
    domain_l1 = dp.numpy.array2_domain(
        num_columns=3, size=2, T=float, norm=B, p=1, nan=False
    )
    domain_l2 = dp.numpy.array2_domain(
        num_columns=3, size=2, T=float, norm=B, p=2, nan=False
    )

    l1_default = make_np_sscp(domain_l1, metric)
    assert l1_default.output_metric == dp.l1_distance(T=float)
    assert l1_default.map(2) == 2 * B**2

    # The matrix metric is independent of the row-clipping norm.
    l1_from_l2 = make_np_sscp(
        domain_l2, metric, output_metric=dp.l1_distance(T=float)
    )
    assert l1_from_l2.map(2) == 2 * 3 * B**2

    l2_from_l1 = make_np_sscp(
        domain_l1, metric, output_metric=dp.l2_distance(T=float)
    )
    assert l2_from_l1.map(1) == 0
    assert l2_from_l1.map(2) >= np.sqrt(2) * B**2

    nuclear = make_np_sscp(
        domain_l1, metric, output_metric=_nuclear_norm_distance()
    )
    assert nuclear.output_metric == _nuclear_norm_distance()
    assert nuclear.map(2) == 2 * B**2

    X = np.array([[B, 0, 0], [0, B, 0]], dtype=float)
    assert np.array_equal(l1_default(X), X.T @ X)
    assert l1_default.output_domain.member(l1_default(X))


def test_np_sscp_rejects_unrecognized_output_metric():
    domain = dp.numpy.array2_domain(
        num_columns=2, T=float, norm=1.0, p=2, nan=False
    )
    with pytest.raises(ValueError, match="output_metric"):
        make_np_sscp(
            domain, dp.symmetric_distance(), output_metric=dp.absolute_distance(T=float)
        )


@pytest.mark.parametrize("p", [1, 2])
def test_np_sscp_scalar_substitution_exact_for_all_output_metrics(p):
    from opendp.extras.numpy import _nuclear_norm_distance

    radius = 2.0
    domain = dp.numpy.array2_domain(
        num_columns=1, size=2, T=float, norm=radius, p=p, nan=False
    )
    metric = dp.symmetric_distance()
    x = np.array([[radius], [0.0]])
    y = np.zeros((2, 1))
    for output_metric in (
        dp.l1_distance(T=float),
        dp.l2_distance(T=float),
        _nuclear_norm_distance(),
    ):
        trans = make_np_sscp(domain, metric, output_metric=output_metric)
        assert trans.map(1) == 0
        assert trans.map(2) == radius**2
        assert trans.map(4) == 2 * radius**2
        assert trans(x)[0, 0] - trans(y)[0, 0] == radius**2
