import opendp.prelude as dp
from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
import pytest

np = pytest.importorskip("numpy")
pytest.importorskip("randomgen")
pytest.importorskip("scipy.linalg")


def test_private_eigenvector():
    from opendp.extras.sklearn._make_eigenvector import then_private_eigenvector

    space = (
        _symmetric_matrix_domain(num_features=4, T=float),
        _nuclear_norm_distance(),
    )
    meas = space >> then_private_eigenvector(concentration=25_000.0)

    data = np.random.normal(size=(4, 4))
    data += data.T
    noisy = meas(data)
    exact = np.linalg.eigh(data)[1][:, -1]

    # normalize sign
    noisy = np.copysign(noisy, exact)

    assert np.allclose(noisy, exact, atol=0.3)
    assert meas.map(2) == 100_000.0
    assert meas.map(1) == 50_000.0


@pytest.mark.parametrize("uniform, expected", [(0.5, [0.0, 1.0]), (0.9, [1.0, 0.0])])
def test_private_eigenvector_acceptance_probability(monkeypatch, uniform, expected):
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    class FixedRNG:
        def __init__(self):
            # eigh orders the eigenvalues of C increasingly: e2, then e1.
            self.proposals = iter((np.array([1.0, 0.0]), np.array([0.0, 1.0])))

        def standard_normal(self, size):
            assert size == 2
            return next(self.proposals)

        def random(self):
            return uniform

    monkeypatch.setattr(eigenvector, "get_np_csprng", FixedRNG)

    space = (
        _symmetric_matrix_domain(num_features=2, T=float),
        _nuclear_norm_distance(),
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
    # At 0.9, the correct sampler also rejects e2, then accepts e1 (~0.95).
    assert np.array_equal(meas(np.diag([1.0, 0.0])), expected)


def test_private_eigenvector_distribution(monkeypatch):
    from scipy.stats import kstest, vonmises  # type: ignore[import-untyped]
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    monkeypatch.setattr(eigenvector, "get_np_csprng", lambda: np.random.default_rng(0))

    space = (
        _symmetric_matrix_domain(num_features=2, T=float),
        _nuclear_norm_distance(),
    )
    meas = space >> eigenvector.then_private_eigenvector(concentration=1.0)

    # With concentration = 1, the Bingham density is proportional to
    # exp(-sin(theta)**2), so 2 * theta follows von Mises(0, 0.5).
    samples = np.array([meas(np.diag([1.0, 0.0])) for _ in range(1024)])
    angles = 2 * np.arctan2(samples[:, 1], samples[:, 0])
    angles = (angles + np.pi) % (2 * np.pi) - np.pi

    assert kstest(angles, vonmises(0.5).cdf).pvalue > 1e-6


@pytest.mark.parametrize("width", [2, 4, 128])
@pytest.mark.parametrize("value", [0.0, 1.0])
def test_private_eigenvector_isotropic_matrix(width, value):
    from opendp.extras.sklearn._make_eigenvector import make_private_eigenvector

    domain = _symmetric_matrix_domain(num_features=width, T=float)
    measurement = make_private_eigenvector(domain, _nuclear_norm_distance(), 0.25)
    vector = measurement(np.eye(width) * value)
    assert np.isfinite(vector).all()
    assert np.linalg.norm(vector) == pytest.approx(1.0)
    assert measurement.map(2) == 1.0


@pytest.mark.parametrize("concentration", [0.0, 2.0])
@pytest.mark.parametrize("dtype", [np.float32, np.float64])
def test_private_eigenvector_eigenbasis(monkeypatch, concentration, dtype):
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    # A dense matrix with a repeated eigenvalue exercises scaling AND rotation.
    Q = np.array([[1, 1, 1, 1], [1, -1, 1, -1],
                  [1, 1, -1, -1], [1, -1, -1, 1]], dtype=dtype) / 2
    C = (Q * np.array([0, 0, 1, 3], dtype=dtype)) @ Q.T
    values, basis = np.linalg.eigh(C)
    a = concentration * (float(values[-1]) - values.astype(float))
    b = eigenvector._optimal_b(a, 4)
    omega = 1 + 2 * a / b
    g = np.array([1.0, 2.0, 3.0, 4.0])
    z = g / np.sqrt(omega)
    expected = basis @ (z / np.linalg.norm(z))
    # Check the diagonal forms against their original-coordinate matrix forms.
    A = concentration * (float(values[-1]) * np.eye(4) - C)
    Omega = np.eye(4) + 2 * A / b
    w_squared = (z / np.linalg.norm(z)) ** 2
    assert a @ w_squared == pytest.approx(expected @ A @ expected, abs=1e-6)
    assert omega @ w_squared == pytest.approx(expected @ Omega @ expected, abs=1e-6)

    class FixedRNG:
        def standard_normal(self, size):
            assert size == 4
            return g

        def random(self):
            return 0.0  # Also exercises the endpoint without taking log(0).

    calls = []
    eigh = np.linalg.eigh

    def counted_eigh(matrix):
        calls.append(matrix)
        return eigh(matrix)

    monkeypatch.setattr(np.linalg, "eigh", counted_eigh)
    monkeypatch.setattr(eigenvector, "get_np_csprng", FixedRNG)
    measurement = eigenvector.make_private_eigenvector(
        _symmetric_matrix_domain(num_features=4, T=dp.f32 if dtype == np.float32 else dp.f64),
        _nuclear_norm_distance(), concentration)
    np.testing.assert_allclose(measurement(C), expected, atol=1e-6)
    assert len(calls) == 1
    assert measurement.map(2) == 4 * concentration


def test_private_eigenvector_log_envelope(monkeypatch):
    import opendp.extras.sklearn._make_eigenvector as eigenvector

    # log(M) > 709: the old exp/power expression overflows. Mock only the
    # eigendecomposition to keep this numerical regression test inexpensive.
    d = 512
    values = np.r_[np.full(d - 1, -1000.0), 0.0]
    monkeypatch.setattr(np.linalg, "eigh", lambda C: (values, np.eye(d)))
    monkeypatch.setattr(eigenvector, "get_np_csprng", lambda: np.random.default_rng(3))
    measurement = eigenvector.make_private_eigenvector(
        _symmetric_matrix_domain(num_features=d), _nuclear_norm_distance(), 1.0)
    with np.errstate(all="raise"):
        vector = measurement(np.diag(values))
    assert np.isfinite(vector).all()
    assert np.linalg.norm(vector) == pytest.approx(1.0)

    # Invalid envelope computations must fail explicitly, not clip or loop.
    monkeypatch.setattr(eigenvector, "_optimal_b", lambda a, d: float("nan"))
    with pytest.raises(dp.OpenDPException, match="invalid Bingham rejection envelope"):
        measurement(np.diag(values))
    # A finite but wrong normalizer that would force p > 1 must also fail.
    monkeypatch.setattr(eigenvector, "_optimal_b", lambda a, d: 1.0)
    monkeypatch.setattr(np, "log1p", lambda x: -100.0)
    with pytest.raises(dp.OpenDPException, match="invalid Bingham log acceptance"):
        measurement(np.diag(values))


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
    concentrations = [1.0 / (4 * 4.0**2)] * 3
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
    with pytest.raises(ValueError, match="nuclear norm"):
        make_private_eigenvector(domain, dp.symmetric_distance(), 1.0)
    with pytest.raises(ValueError, match="finite and non-negative"):
        make_private_eigenvector(domain, _nuclear_norm_distance(), -1.0)


def test_projected_sscp_has_identity_nuclear_stability():
    from opendp.extras.sklearn._make_eigenvector import make_np_sscp_projection

    domain = _symmetric_matrix_domain(num_features=2)
    metric = _nuclear_norm_distance()
    P = np.array([[1.0, 0.0]])
    trans = make_np_sscp_projection(domain, metric, P)
    assert trans.map(2.0) == 2.0
    P[:] = 10.0  # Stored projection must not share this mutable array.
    np.testing.assert_array_equal(trans(np.diag([3.0, 4.0])), [[3.0]])


def test_projection_promotes_float32_input_to_float64_output():
    from opendp.extras.sklearn._make_eigenvector import make_np_sscp_projection

    domain = _symmetric_matrix_domain(num_features=2, T=dp.f32)
    P = np.array([[1.0, 0.0]], dtype=np.float64)
    trans = make_np_sscp_projection(domain, _nuclear_norm_distance(), P)
    matrix = np.diag([3.0, 4.0]).astype(np.float32)
    assert trans.output_domain.member(trans(matrix))
    assert trans(matrix).dtype == np.float64
