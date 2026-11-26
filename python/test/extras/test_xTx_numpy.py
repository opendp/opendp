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
    assert trans.map(1) == (100.0, 100.0)


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
    trans = make_np_sscp(domain, dp.symmetric_distance())
    data = np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)

    assert domain.member(data)
    actual = trans(data)
    assert np.array_equal(actual, data.T @ data)
    assert actual.dtype == data.dtype
    assert trans.output_domain.member(actual)
    assert trans.map(1) == (0.0, 0.0)


@pytest.mark.parametrize("norm", [None, float("nan"), float("inf")])
def test_np_sscp_requires_finite_norm(norm):
    kwargs = {} if norm is None else {"norm": norm, "p": 2}
    domain = dp.numpy.array2_domain(num_columns=2, T=float, nan=False, **kwargs)
    with pytest.raises(ValueError, match="finite row norm"):
        make_np_sscp(domain, dp.symmetric_distance())


def test_np_sscp_joint_sensitivity():
    B = 2.0
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=float, norm=B, p=2, nan=False
    )
    trans = make_np_sscp(domain, dp.symmetric_distance())

    assert trans.output_metric == dp.schatten12_distance()
    assert trans.map(1) == (0.0, 0.0)
    assert trans.map(2) == pytest.approx((2 * B**2, np.sqrt(2) * B**2))

    x = np.array([B, 0.0])
    y = np.array([0.0, B])
    delta = np.outer(x, x) - np.outer(y, y)
    nuclear_distance = np.linalg.svd(delta, compute_uv=False).sum()
    assert nuclear_distance == trans.map(2)[0]
    assert np.linalg.norm(delta, ord="fro") <= trans.map(2)[1]
    assert trans.map(3) == trans.map(2)


def test_np_sscp_frobenius_substitution_sensitivity():
    norm = 2.0
    fixed = dp.numpy.array2_domain(
        num_columns=2, size=3, T=float, norm=norm, p=2, nan=False
    )
    metric = dp.symmetric_distance()
    trans = make_np_sscp(fixed, metric, output_metric=dp.l2_distance(T=float))
    assert trans.output_metric == dp.l2_distance(T=float)
    assert trans.map(1) == 0
    assert trans.map(2) >= np.sqrt(2) * norm**2
    assert trans.map(2) < 2 * norm**2

    x = np.array([norm, 0.0])
    y = np.array([0.0, norm])
    delta = np.outer(x, x) - np.outer(y, y)
    assert np.linalg.norm(delta, ord="fro") <= trans.map(2)

    unsized = dp.numpy.array2_domain(num_columns=2, T=float, norm=norm, p=2, nan=False)
    generic = make_np_sscp(unsized, metric, output_metric=dp.l2_distance(T=float))
    assert generic.map(2) == 2 * norm**2


def test_np_sscp_explicit_l1_and_independent_output_metrics():
    B = 2.0
    metric = dp.symmetric_distance()
    domain_l1 = dp.numpy.array2_domain(
        num_columns=3, size=2, T=float, norm=B, p=1, nan=False
    )
    domain_l2 = dp.numpy.array2_domain(
        num_columns=3, size=2, T=float, norm=B, p=2, nan=False
    )

    l1_from_l1 = make_np_sscp(domain_l1, metric, output_metric=dp.l1_distance(T=float))
    assert l1_from_l1.output_metric == dp.l1_distance(T=float)
    assert l1_from_l1.map(2) == 2 * B**2

    # The matrix metric is independent of the row-clipping norm.
    l1_from_l2 = make_np_sscp(domain_l2, metric, output_metric=dp.l1_distance(T=float))
    assert l1_from_l2.map(2) == 2 * 3 * B**2

    l2_from_l1 = make_np_sscp(domain_l1, metric, output_metric=dp.l2_distance(T=float))
    assert l2_from_l1.map(1) == 0
    assert l2_from_l1.map(2) >= np.sqrt(2) * B**2

    nuclear = make_np_sscp(domain_l1, metric, output_metric=dp.schatten12_distance())
    assert nuclear.output_metric == dp.schatten12_distance()
    assert nuclear.map(2) == pytest.approx((2 * B**2, np.sqrt(2) * B**2))

    X = np.array([[B, 0, 0], [0, B, 0]], dtype=float)
    assert np.array_equal(l1_from_l1(X), X.T @ X)
    assert l1_from_l1.output_domain.member(l1_from_l1(X))


def test_np_sscp_rejects_unrecognized_output_metric():
    domain = dp.numpy.array2_domain(num_columns=2, T=float, norm=1.0, p=2, nan=False)
    with pytest.raises(ValueError, match="output_metric"):
        make_np_sscp(
            domain, dp.symmetric_distance(), output_metric=dp.absolute_distance(T=float)
        )


@pytest.mark.parametrize("p", [1, 2])
def test_np_sscp_scalar_substitution_exact_for_all_output_metrics(p):
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
        dp.schatten12_distance(),
    ):
        trans = make_np_sscp(domain, metric, output_metric=output_metric)
        if output_metric == dp.schatten12_distance():
            assert trans.map(1) == (0.0, 0.0)
            assert trans.map(2) == (radius**2, radius**2)
            assert trans.map(4) == (2 * radius**2, 2 * radius**2)
        else:
            assert trans.map(1) == 0
            assert trans.map(2) == radius**2
            assert trans.map(4) == 2 * radius**2
        assert trans(x)[0, 0] - trans(y)[0, 0] == radius**2


@pytest.mark.parametrize(
    ("norm", "num_columns", "size", "metric_kind", "error"),
    [
        (1e-200, 2, None, "nuclear", "row norm squared underflows"),
        (1e200, 2, None, "nuclear", "row norm squared overflows"),
        (1e154, 2, None, "l1", "SSCP sensitivity overflows"),
        (1.2e154, 2, 2, "l2", "Frobenius replacement sensitivity overflows"),
    ],
)
def test_np_sscp_rejects_unrepresentable_sensitivity(
    norm, num_columns, size, metric_kind, error
):
    domain = dp.numpy.array2_domain(
        num_columns=num_columns, size=size, T=float, norm=norm, p=2, nan=False
    )
    output_metrics = {
        "nuclear": None,
        "l1": dp.l1_distance(T=float),
        "l2": dp.l2_distance(T=float),
    }
    with pytest.raises(ValueError, match=error):
        make_np_sscp(
            domain,
            dp.symmetric_distance(),
            output_metric=output_metrics[metric_kind],
        )


@pytest.mark.parametrize("norm", [2.0, 1e154, 1.2e154])
def test_np_sscp_frobenius_overflow_with_strict_numpy_errors(norm):
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=float, norm=norm, p=2, nan=False
    )
    with np.errstate(over="raise"):
        if norm == 1.2e154:
            with pytest.raises(ValueError, match="Frobenius replacement sensitivity"):
                make_np_sscp(domain, dp.symmetric_distance(), dp.l2_distance(T=float))
        else:
            trans = make_np_sscp(
                domain, dp.symmetric_distance(), dp.l2_distance(T=float)
            )
            expected = np.nextafter(np.sqrt(2.0) * (norm * norm), np.inf)
            assert np.isfinite(trans.map(2))
            assert trans.map(2) == expected
            assert trans.map(1) == 0
        assert np.geterr()["over"] == "raise"


@pytest.mark.parametrize("T, dtype", [(dp.f32, np.float32), (dp.f64, np.float64)])
@pytest.mark.parametrize("explicit", [False, True])
def test_joint_sscp_does_not_construct_native_lp_metrics(
    T, dtype, explicit, monkeypatch
):
    domain = dp.numpy.array2_domain(
        num_columns=2, size=2, T=T, norm=1.0, p=2, nan=False
    )
    joint_metric = dp.schatten12_distance()

    def unexpected(*args, **kwargs):
        raise AssertionError("Joint SSCP must not construct unrelated Lp metrics")

    monkeypatch.setattr(dp, "l1_distance", unexpected)
    monkeypatch.setattr(dp, "l2_distance", unexpected)
    trans = make_np_sscp(
        domain, dp.symmetric_distance(), joint_metric if explicit else None
    )
    data = np.array([[0.5, 0.0], [0.5, 0.5]], dtype=dtype)
    result = trans(data)
    np.testing.assert_array_equal(result, data.T @ data)
    assert result.dtype == dtype
    assert trans.output_domain.member(result)
    assert trans.output_metric == joint_metric
    assert trans.map(1) == (0.0, 0.0)
    assert trans.map(2) == pytest.approx((2.0, np.sqrt(2.0)))
