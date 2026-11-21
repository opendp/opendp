from typing import Sequence
from dataclasses import asdict

from opendp.extras.numpy import _nuclear_norm_distance, _symmetric_matrix_domain
from opendp.extras._utilities import to_then
from opendp._lib import get_np_csprng, import_optional_dependency
from opendp.mod import Domain, Metric, Transformation, Measurement
from opendp._internal import _make_measurement, _make_transformation


def _optimal_b(A_eigvals, d: int) -> float:
    import opendp.prelude as dp

    # Differs from the Amin et al. in two ways:
    # 1. In 3.6 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf,
    #   the equality is against 1 not 0
    # 2. Instead of using bounds of (1, d), widen both bounds by a margin,
    #   as the boundary may lie on a bound: numerically at 1, and exactly at d when A = 0
    margin = 0.1
    return dp.binary_search(
        lambda b: sum(1 / (b + 2 * A_eigvals)) >= 1,
        bounds=(1.0 - margin, float(d) + margin),
    )


# planning to make this public, but may make more API changes


def make_private_eigenvector(
    input_domain: Domain, input_metric: Metric, concentration: float
) -> Measurement:
    """Construct a Measurement that releases a private eigenvector from a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear norm distance on symmetric matrices
    :param concentration: coefficient of u.T @ C @ u in the Bingham density.
        The privacy map is concentration times the nuclear-norm distance.
    """
    np = import_optional_dependency("numpy")
    assert np is not None
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    np_csprng = get_np_csprng()
    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != _nuclear_norm_distance():
        raise ValueError("expected nuclear norm distance input metric")

    if not np.isfinite(concentration) or concentration < 0:
        raise ValueError("concentration must be finite and non-negative")

    input_desc = input_domain.descriptor
    d = input_desc.num_features

    def function(C):
        # Algorithm 2 Top Eigenvector Sampler
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=15
        # (1) Shifting the score by its largest eigenvalue does not
        # change the target Bingham distribution.
        eigvals, Q = np.linalg.eigh(C)
        # Work in the eigenbasis, widening float32 eigenvalues before arithmetic.
        eigvals = eigvals.astype(float)
        with np.errstate(over="raise", invalid="raise"):
            A_eigvals = concentration * (eigvals[-1] - eigvals)

        # This algorithm first finds parameters for an angular central gaussian distribution (ACG),
        #   which acts as an envelope distribution for rejection sampling.
        # The rejection rate will be minimized when b is chosen optimally.
        # b is chosen optimally when the ACG is least entropic, but is still an envelope.
        # Criteria for being an envelope are given in 3.4 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf

        b = _optimal_b(A_eigvals, d)
        roundoff = 64 * np.finfo(float).eps
        if not np.isfinite(b) or not 0 < b <= d * (1 + roundoff):
            raise ValueError("invalid Bingham rejection envelope parameter b")
        with np.errstate(over="raise", invalid="raise", divide="raise"):
            omega = 1 + 2 * A_eigvals / b
            # log(M), using log1p for accuracy when b is close to d.
            log_M = -(d - b) / 2 + (d / 2) * np.log1p((d - b) / b)
            proposal_scale = 1 / np.sqrt(omega)
        if not np.isfinite(omega).all() or not np.isfinite(log_M):
            raise ValueError("non-finite Bingham rejection envelope")

        while True:
            # This continuous sampler still requires idealized numerics:
            # normal sampling, normalization and rejection are not floating-point-safe.
            z = np_csprng.standard_normal(d) * proposal_scale  # type: ignore[union-attr]
            norm = np.linalg.norm(z)
            if not np.isfinite(norm) or norm == 0:
                raise ValueError("invalid Bingham proposal norm")
            # w is the ACG proposal in the eigenbasis.
            w = z / norm
            w_squared = w * w
            score = A_eigvals @ w_squared
            # Amin et al., Algorithm 2 step 4(b), incorrectly divides by
            # (u.T @ Omega @ u) ** (d / 2). The ACG proposal density
            # contains its inverse, so rejection sampling must multiply
            # by this factor instead (Kent et al., Eq. 3.3).
            log_acg = (d / 2) * np.log(omega @ w_squared)
            log_p = -score + log_acg - log_M
            # Allow cancellation-scale roundoff, but do not clip invalid envelopes
            # to probability one or silently retry NaNs indefinitely.
            # The log argument's rounding error is amplified by d/2, even
            # when its logarithm is near zero (notably for isotropic inputs).
            tolerance = roundoff * max(float(d), abs(score), abs(log_acg), abs(log_M))
            if not np.isfinite(log_p) or log_p > tolerance:
                raise ValueError("invalid Bingham log acceptance probability")
            uniform = np_csprng.random()
            if uniform == 0 or np.log(uniform) < log_p:
                # Only accepted proposals pay for the O(d^2) rotation.
                return (Q @ w).astype(C.dtype, copy=False)

    return _make_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(),
        function,
        # For Δ = C - C', spectral_range(Δ) <= ||Δ||_*.
        # Bounding the likelihood ratio via the normalizing constants
        # Conservative Bingham bound: two factors of exp(κ ||Δ||_*)
        # from the score and its normalizer.
        lambda d_in: 2 * concentration * d_in,
        TO=f"Vec<{input_desc.T}>",
    )


# generate then variant of the constructor
then_private_eigenvector = to_then(make_private_eigenvector)


def make_np_sscp_projection(
    input_domain: Domain, input_metric: Metric, P
) -> Transformation:
    """Construct a Transformation that projects an SSCP matrix.

    The operator norm of `P` must be at most one. Under this
    precondition, ||P @ (C - C') @ P.T||_* <= ||C - C'||_*.
    The condition is a precondition, not an SVD checked at runtime.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear norm distance on symmetric matrices
    :param P: a linear map with singular values no greater than 1
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != _nuclear_norm_distance():
        raise ValueError("expected nuclear norm distance input metric")

    input_desc = input_domain.descriptor
    # Retain a private copy so that later caller mutation cannot change the map.
    P = np.array(P, copy=True)
    if P.ndim != 2 or not np.isfinite(P).all():
        raise ValueError("P must be a finite 2d matrix")
    if P.dtype not in {np.dtype("float32"), np.dtype("float64")}:
        raise ValueError("P must have a real float32 or float64 dtype")

    if input_desc.num_features != P.shape[1]:
        raise ValueError(
            f"projection P (axis-1 size: {P.shape[1]}) does not conform with data in input_domain (num_features: {input_domain.num_features})"
        )  # pragma: no cover

    # Preserve the dtype produced by matrix multiplication (which can widen
    # float32 SSCP matrices when an orthonormal basis has float64 entries).
    output_T = dp.f64 if input_desc.T == dp.f64 or P.dtype == np.float64 else dp.f32
    kwargs = asdict(input_desc) | {"num_features": P.shape[0], "T": output_T}
    return _make_transformation(
        input_domain,
        input_metric,
        _symmetric_matrix_domain(**kwargs),
        input_metric,
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5
        # Algorithm 1 step 2.c
        lambda cov: P @ cov @ P.T,  # c: update the covariance matrix
        lambda d_in: d_in,
    )


# generate then variant of the constructor
then_np_sscp_projection = to_then(make_np_sscp_projection)


def make_private_eigenvectors(
    input_domain: Domain, input_metric: Metric, concentrations: Sequence[float]
) -> Measurement:
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    linalg = import_optional_dependency("scipy.linalg")

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != _nuclear_norm_distance():
        raise ValueError("expected nuclear norm distance input metric")

    input_desc = input_domain.descriptor
    if len(concentrations) > input_desc.num_features - 1:
        raise ValueError(
            f"must specify at most {input_desc.num_features - 1} concentrations"
        )
    if any(not np.isfinite(c) or c < 0 for c in concentrations):
        raise ValueError("concentrations must be finite and non-negative")

    privacy_measure = dp.max_divergence()
    m_compose = dp.c.make_adaptive_composition(
        input_domain, input_metric, privacy_measure, 1.0,
        [2 * concentration for concentration in concentrations],
    )

    def function(C):
        # Algorithm 1: http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5

        # 1.i Initialize C_1 = C inside compositor.
        #     C_i will be computed by `_make_np_sscp_projection`
        qbl = m_compose(C)
        del C  # only the compositor has the data now

        # 1.ii initialize P_1 with the identity matrix
        P = np.eye(input_desc.num_features)
        theta = np.zeros((0, input_desc.num_features))

        # computation of eigenvalues (1.iii) happens in a separate constructor

        # 2. only runs until epsilons are exhausted, not until num_features
        for concentration_i in concentrations:
            # 2.a.i: sample \hat{u}
            m_eigvec = (
                (input_domain, input_metric)
                >> then_np_sscp_projection(P)  # 2.c happens inside this transformation
                >> then_private_eigenvector(concentration_i)
                >> dp.as_array()
            )
            u = qbl(m_eigvec)

            # 2.a.ii: extend the set of eigenvectors
            theta = np.vstack((theta, P.T @ u))

            # 2.b: update the projection, P maintains singular values with magnitude <= 1
            P = linalg.null_space(theta).T

        # the eigvec of a 1x1 matrix is always I, so it doesn't need to be released
        # so if down to the last eigvec, return the projection built up via postprocessing
        if P.shape[0] == 1:
            theta = np.vstack((theta, P))

        return theta.T

    return _make_measurement(
        input_domain,
        input_metric,
        m_compose.output_measure,
        function,
        # Each adaptive query has privacy slope 2 * concentration_i in
        # nuclear-norm distance; adaptive composition sums the slopes.
        lambda d_in: d_in * m_compose.map(1.0),
    )


# generate then variant of the constructor
then_private_eigenvectors = to_then(make_private_eigenvectors)
