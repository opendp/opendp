import opendp.prelude as dp
from opendp.extras.numpy import _symmetric_matrix_domain
import pytest

np = pytest.importorskip("numpy")
pytest.importorskip("randomgen")
pytest.importorskip("scipy.linalg")


def test_private_eigenvector():
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvector

    space = (
        _symmetric_matrix_domain(num_features=4, T=float),
        dp.nuclear_distance(),
    )
    meas = space >> then_private_eigenvector(concentration=25_000.0)

    data = np.random.normal(size=(4, 4))
    data += data.T
    noisy = meas(data)
    exact = np.linalg.eigh(data)[1][:, -1]

    # normalize sign
    noisy = np.copysign(noisy, exact)

    assert np.allclose(noisy, exact, atol=0.3)
    assert meas.map(2.0) == 50_000.0
    assert meas.map(1.0) == 25_000.0


def test_private_eigenvector_acceptance_probability(monkeypatch):
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    class FixedRNG:
        def __init__(self):
            self.proposals = iter((np.array([0.0, 1.0]), np.array([1.0, 0.0])))

        def multivariate_normal(self, mean, cov):
            return next(self.proposals)

        def random(self):
            return 0.5

    monkeypatch.setattr(eigenvector, "get_np_csprng", FixedRNG)

    space = (
        _symmetric_matrix_domain(num_features=2, T=float),
        dp.nuclear_distance(),
    )
    meas = space >> eigenvector.then_private_eigenvector(concentration=1.0)

    # For C = diag(1, 0) and concentration = 1:
    #   A = diag(0, 1), b = sqrt(2), Omega = diag(1, 1 + sqrt(2)).
    #
    # The first proposal, u = e2, has acceptance probability ~0.84
    # under the correct Bingham rejection sampler, but only ~0.14
    # under the inverted formula in Amin et al., Algorithm 2.
    #
    # Fixing the uniform draw at 0.5 makes the test deterministic:
    # the correct sampler accepts e2, whereas the buggy sampler
    # rejects it and proceeds to the second proposal, e1.
    assert np.array_equal(meas(np.diag([1.0, 0.0])), [0.0, 1.0])


def test_private_eigenvector_distribution(monkeypatch):
    from scipy.stats import kstest, vonmises  # type: ignore[import-untyped]
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    monkeypatch.setattr(eigenvector, "get_np_csprng", lambda: np.random.default_rng(0))

    space = (
        _symmetric_matrix_domain(num_features=2, T=float),
        dp.nuclear_distance(),
    )
    meas = space >> eigenvector.then_private_eigenvector(concentration=1.0)

    # With concentration = 1, the Bingham density is proportional to
    # exp(-sin(theta)**2), so 2 * theta follows von Mises(0, 0.5).
    samples = np.array([meas(np.diag([1.0, 0.0])) for _ in range(1024)])
    angles = 2 * np.arctan2(samples[:, 1], samples[:, 0])
    angles = (angles + np.pi) % (2 * np.pi) - np.pi

    assert kstest(angles, vonmises(0.5).cdf).pvalue > 1e-6


@pytest.mark.parametrize("width", [2, 4])
@pytest.mark.parametrize("value", [0.0, 1.0])
@pytest.mark.parametrize("dtype", [np.float32, np.float64])
def test_private_eigenvector_isotropic_matrix(width, value, dtype):
    from opendp.extras.sklearn._make_eigenvector import make_private_eigenvector

    domain = _symmetric_matrix_domain(
        num_features=width, T=dp.f32 if dtype == np.float32 else dp.f64
    )
    measurement = make_private_eigenvector(domain, dp.nuclear_distance(), 0.25)
    vector = measurement(np.eye(width, dtype=dtype) * value)
    assert np.isfinite(vector).all()
    assert np.linalg.norm(vector) == pytest.approx(1.0)
    assert measurement.map(2.0) == 0.5


def test_eigenvector_integration():
    from opendp.extras.numpy import then_np_clamp
    from opendp.extras.numpy._make_np_sscp import then_np_sscp
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvector

    num_columns = 4
    domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = (
        domain,
        dp.symmetric_distance(),
    )
    meas = (
        space
        >> then_np_clamp(norm=1.0, p=2)
        >> then_np_sscp()
        >> then_private_eigenvector(1.0)
    )

    data = np.random.normal(size=(1000, num_columns))
    print("meas(data)", meas(data))


def test_eigenvectors():
    from opendp.extras.numpy import then_np_clamp
    from opendp.extras.numpy._make_np_sscp import then_np_sscp
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvectors

    num_columns = 4
    domain = dp.numpy.array2_domain(num_columns=num_columns, T=float)
    space = (
        domain,
        dp.symmetric_distance(),
    )
    sp_sscp = space >> then_np_clamp(norm=4.0, p=2) >> then_np_sscp()
    # Row norm B = 4, one replacement has nuclear sensitivity 2B^2.
    concentrations = [1.0 / (2 * 4.0**2)] * 3
    meas = sp_sscp >> then_private_eigenvectors(concentrations)
    assert meas.map(2) == pytest.approx(3.0)

    data = np.random.normal(size=(1000, num_columns))
    print("meas(data)", meas(data))


def test_optimal_b_zero():
    from opendp.extras.sklearn._make_eigenvector import _optimal_b

    # A = 0 when the data is empty
    assert _optimal_b(np.zeros(4), 4) == pytest.approx(4)


def test_private_eigenvector_rejects_wrong_metric_and_bad_concentration():
    from opendp.extras.sklearn._make_eigenvector import make_private_eigenvector

    domain = _symmetric_matrix_domain(num_features=2)
    with pytest.raises(ValueError, match="nuclear"):
        make_private_eigenvector(domain, dp.symmetric_distance(), 1.0)
    with pytest.raises(ValueError, match="finite and non-negative"):
        make_private_eigenvector(domain, dp.nuclear_distance(), -1.0)


def test_projected_sscp_has_identity_nuclear_stability():
    from opendp.extras.sklearn._make_eigenvector import _make_np_sscp_projection

    domain = _symmetric_matrix_domain(num_features=2)
    metric = dp.nuclear_distance()
    P = np.array([[1.0, 0.0]])
    trans = _make_np_sscp_projection(domain, metric, P)
    assert trans.map(2.0) == 2.0
    P[:] = 10.0  # Stored projection must not share this mutable array.
    np.testing.assert_array_equal(trans(np.diag([3.0, 4.0])), [[3.0]])


def test_projection_promotes_float32_input_to_float64_output():
    from opendp.extras.sklearn._make_eigenvector import _make_np_sscp_projection

    domain = _symmetric_matrix_domain(num_features=2, T=dp.f32)
    P = np.array([[1.0, 0.0]], dtype=np.float64)
    trans = _make_np_sscp_projection(domain, dp.nuclear_distance(), P)
    matrix = np.diag([3.0, 4.0]).astype(np.float32)
    assert trans.output_domain.member(trans(matrix))
    assert trans(matrix).dtype == np.float64


def test_nuclear_eigenvector_rejects_invalid_spaces():
    from opendp.extras.sklearn._make_eigenvector import (
        _make_np_sscp_projection,
        make_private_eigenvector,
        make_private_eigenvectors,
    )

    matrix = _symmetric_matrix_domain(num_features=2)
    invalid = dp.numpy.array2_domain(num_columns=2, T=float)
    metric = dp.nuclear_distance()
    incorrect_metric = dp.l2_distance(T=float)
    with pytest.raises(ValueError, match="NPSymmetricMatrixDomain"):
        make_private_eigenvector(invalid, metric, 0.0)
    with pytest.raises(ValueError, match="NPSymmetricMatrixDomain"):
        _make_np_sscp_projection(invalid, metric, np.eye(2))
    with pytest.raises(ValueError, match="nuclear"):
        _make_np_sscp_projection(matrix, incorrect_metric, np.eye(2))
    with pytest.raises(ValueError, match="NPSymmetricMatrixDomain"):
        make_private_eigenvectors(invalid, metric, [])
    with pytest.raises(ValueError, match="nuclear"):
        make_private_eigenvectors(matrix, incorrect_metric, [])
    with pytest.raises(ValueError, match="at most"):
        make_private_eigenvectors(matrix, metric, [0.0, 0.0])
    with pytest.raises(ValueError, match="non-negative"):
        make_private_eigenvectors(matrix, metric, [float("nan")])


@pytest.mark.parametrize(
    "projection, message",
    [
        (np.array([1.0, 0.0]), "finite 2d"),
        (np.array([[float("nan"), 0.0]]), "finite 2d"),
        (np.array([[1, 0]]), "float32 or float64"),
    ],
)
def test_nuclear_projection_rejects_invalid_matrix(projection, message):
    from opendp.extras.sklearn._make_eigenvector import _make_np_sscp_projection

    domain = _symmetric_matrix_domain(num_features=2)
    with pytest.raises(ValueError, match=message):
        _make_np_sscp_projection(domain, dp.nuclear_distance(), projection)


def test_nuclear_eigenvector_composition_map():
    from opendp.extras.sklearn._make_eigenvector import make_private_eigenvectors

    measurement = make_private_eigenvectors(
        _symmetric_matrix_domain(num_features=3),
        dp.nuclear_distance(),
        [1.0, 2.0],
    )
    assert measurement.map(0.0) == 0.0
    assert measurement.map(2.0) == pytest.approx(6.0)
    assert measurement.map(5.0) == pytest.approx(15.0)
    assert measurement(np.eye(3)).shape == (3, 3)
