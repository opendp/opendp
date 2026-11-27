import math

import pytest
import opendp.prelude as dp

np = pytest.importorskip("numpy")

from opendp._internal import _make_transformation
from opendp.extras.numpy._make_np_moments import (
    _joint_sensitivity,
    make_np_joint_moments,
)


@pytest.mark.parametrize("p", [1, 2])
@pytest.mark.parametrize("b", [0.1, 0.5, 1.0, 3.0])
def test_scalar_joint_moment_sensitivity(p, b):
    x = np.linspace(-1.0, 1.0, 501)
    gram = x[:, None] ** 2 - x[None, :] ** 2
    cross = b * (x[:, None] - x[None, :])
    distances = np.abs(gram) + np.abs(cross) if p == 1 else np.hypot(gram, cross)
    bound = _joint_sensitivity(1.0, b, 1, True, p)
    assert distances.max() <= bound + 1e-12
    assert distances.max() == pytest.approx(bound, abs=2e-5)


@pytest.mark.parametrize(
    "size,p,units,target,q,bound",
    [
        (None, 1, None, True, 0.25, 2.0),
        (None, 2, None, False, 0.25, math.sqrt(2)),
        (1, 1, None, False, 0.25, 5 * math.sqrt(2) / 4),
        (1, 2, None, True, 0.25, 1.25),
        (1, 2, (1, 3), False, 1 / 18, math.sqrt(2 + 2 / 9 + 0.5 / 81)),
        (1, 2, (math.sqrt(2), 1.5), True, 4 / 9, 13 / 9),
        (1, 2, (1, 1 / 3), False, 1, 6.0),
    ],
)
def test_joint_moments(size, p, units, target, q, bound):
    dp.enable_features("contrib", "idealized-numerics")
    domain = dp.numpy.array2_domain(
        T=float, num_columns=2, size=size, norm=1.0, p=2, nan=False
    )
    metric = dp.symmetric_distance()
    features = _make_transformation(
        domain, metric, domain, metric, lambda x: x, lambda d: d
    )
    get_target = (lambda X: np.where(X[:, 1] > 0, -3.0, 3.0)) if target else None
    joint, gram_unit, cross_unit = make_np_joint_moments(
        features, target=get_target, p=p, units=units
    )
    x = np.array([[1.0, 0.0]])
    z = np.array([[q if target else -q, math.sqrt(1 - q * q)]])
    for row, y in [(x, 1), (z, -1 if target else 1)]:
        release = np.asarray(joint(row))
        u, v = row[0]
        np.testing.assert_allclose(
            release[:3] * gram_unit, [u * u, math.sqrt(2) * u * v, v * v]
        )
        np.testing.assert_allclose(release[3:] * cross_unit, row[0] * y)
    neighbor = z if size else x[:0]
    edits = 2 if size else 1
    distance = np.linalg.norm(np.asarray(joint(x)) - joint(neighbor), ord=p)
    assert joint.map(edits) == pytest.approx(bound)
    assert distance <= joint.map(edits) + 1e-12
    if p == 2:
        assert distance == pytest.approx(bound)
    assert joint.map(1 if size else 0) == 0


@pytest.mark.parametrize(
    "radius,target_bound,size,p,error",
    [
        (2.0, 1e308, None, 2, "not representable"),
        (1.0, 1.5e308, None, 1, "Cross-product sensitivity overflows"),
        (1.0, 1e308, 1, 2, "Statistic units"),
    ],
)
def test_joint_moments_unrepresentable_units(radius, target_bound, size, p, error):
    dp.enable_features("contrib", "idealized-numerics")
    domain = dp.numpy.array2_domain(
        T=float, num_columns=2, size=size, norm=radius, p=2, nan=False
    )
    metric = dp.symmetric_distance()
    features = _make_transformation(
        domain, metric, domain, metric, lambda x: x, lambda d: d
    )
    with pytest.raises(ValueError, match=error):
        make_np_joint_moments(features, target_bound=target_bound, p=p)
