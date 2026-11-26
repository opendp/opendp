from collections.abc import Sequence
from dataclasses import asdict

from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras._utilities import to_then
from opendp._lib import get_np_csprng, import_optional_dependency
from opendp.mod import Domain, Measure, Metric, Transformation, Measurement
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


def _score_range(distance):
    # Spectral diameter is bounded by both the nuclear norm and sqrt(2)
    # times the Frobenius norm.
    a, b = distance
    return min(a, 2.0**0.5 * b)


def make_private_eigenvector(
    input_domain: Domain, input_metric: Metric, concentration: float
) -> Measurement:
    """Construct a Measurement that releases a private eigenvector from a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: joint nuclear/Frobenius distance on symmetric matrices
    :param concentration: coefficient of u.T @ C @ u in the Bingham density.
        The privacy map is concentration times the bound on spectral diameter.
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    np_csprng = get_np_csprng()
    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != dp.schatten12_distance():
        raise ValueError("expected joint nuclear/Frobenius distance input metric")

    if not np.isfinite(concentration) or concentration < 0:
        raise ValueError("concentration must be finite and non-negative")

    input_desc = input_domain.descriptor
    d = input_desc.num_features

    def function(C):
        # Algorithm 2 Top Eigenvector Sampler
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=15
        # (1) Shifting the score by its largest eigenvalue does not
        # change the target Bingham distribution.
        A = concentration * (np.linalg.eigvalsh(C).max() * np.eye(d) - C)

        # (2)
        A_eigvals = np.linalg.eigvalsh(A)

        # This algorithm first finds parameters for an angular central gaussian distribution (ACG),
        #   which acts as an envelope distribution for rejection sampling.
        # The rejection rate will be minimized when b is chosen optimally.
        # b is chosen optimally when the ACG is least entropic, but is still an envelope.
        # Criteria for being an envelope are given in 3.4 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf

        b = _optimal_b(A_eigvals, d)
        Omega = np.eye(d) + 2 * A / b

        # (3)
        M = np.exp(-(d - b) / 2) * (d / b) ** (d / 2)

        # prepare sampling
        Omega_inv = np.linalg.inv(Omega)

        while True:
            # Mike Shoemate: I know of no floating-point-safe sampler for `u`.
            #  - Mult normal does not have an invertible cdf
            #  - Could try:
            #    1. a "conservative" Cholesky decomposition of Omega_inv
            #    2. compute clamp_norm(compute L @ std_gaussian(d), 1) with arbitrary precision
            #       sample with sufficient precision where all components round to same float

            z = np_csprng.multivariate_normal(mean=np.zeros(d), cov=Omega_inv)  # type: ignore[union-attr]
            # u is a sample from the angular central gaussian distribution,
            #    an envelope for the bingham distribution
            u = z / np.linalg.norm(z)
            # Amin et al., Algorithm 2 step 4(b), incorrectly divides by
            # (u.T @ Omega @ u) ** (d / 2). The ACG proposal density
            # contains its inverse, so rejection sampling must multiply
            # by this factor instead (Kent et al., Eq. 3.3).
            if np_csprng.random() < (
                np.exp(-u.T @ A @ u) * (u.T @ Omega @ u) ** (d / 2) / M
            ):
                return u

    return _make_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(),
        function,
        # The log-normalizer lies between the minimum and maximum score
        # difference. Hence absolute privacy loss is at most the score range.
        lambda d_in: concentration * _score_range(d_in),
        TO="Vec<f64>",
    )


# generate then variant of the constructor
then_private_eigenvector = to_then(make_private_eigenvector)


def _make_np_sscp_projection(
    input_domain: Domain, input_metric: Metric, P
) -> Transformation:
    """Construct a Transformation that projects an SSCP matrix.

    Internal helper: the caller must supply a contraction (operator norm <= 1).
    In eigendecomposition, P consists of an orthonormal basis of the remaining
    subspace (or the identity). Both the nuclear and Frobenius norms contract under this projection.
    This precondition is not checked with an SVD in the inner loop; it follows
    from construction under the idealized-numerics assumption.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: joint nuclear/Frobenius distance on symmetric matrices
    :param P: a linear map with singular values no greater than 1
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != dp.schatten12_distance():
        raise ValueError("expected joint nuclear/Frobenius distance input metric")

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
_then_np_sscp_projection = to_then(_make_np_sscp_projection)


def make_private_eigenvectors(
    input_domain: Domain,
    input_metric: Metric,
    output_measure: Measure,
    scale: float,
    weights: Sequence[float],
) -> Measurement:
    """Release leading eigenvectors (columns) of a symmetric matrix.

    The scale is the inverse of the sum of the Bingham concentrations.
    Weights give relative concentrations, one per requested component in
    leading-component order. They must be finite and nonnegative.
    At least one sampled vector must have positive weight, if any are sampled.
    The last vector of a full basis follows by postprocessing; its weight
    is ignored because it consumes no privacy expenditure.
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    linalg = import_optional_dependency("scipy.linalg")

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != dp.schatten12_distance():
        raise ValueError("expected joint nuclear/Frobenius distance input metric")

    if output_measure != dp.max_divergence():
        raise ValueError("eigenvectors currently support max_divergence() only")
    if not np.isfinite(scale) or scale <= 0:
        raise ValueError("scale must be positive and finite")

    input_desc = input_domain.descriptor
    dimension = input_desc.num_features
    weights = np.asarray(weights, dtype=float)
    if (
        weights.ndim != 1
        or not 1 <= len(weights) <= dimension
        or not np.isfinite(weights).all()
        or np.any(weights < 0)
    ):
        raise ValueError(
            "weights must be finite, nonnegative, and match the component count"
        )
    num_components = len(weights)
    weights = weights[: dimension - 1]
    if len(weights):
        normalizer = np.linalg.norm(weights, ord=1)
        if not np.isfinite(normalizer) or normalizer <= 0:
            raise ValueError("sampled weights must have a positive finite norm")
        concentrations = (weights / normalizer / scale).tolist()
    else:
        concentrations = []
    if any(not np.isfinite(c) for c in concentrations):
        raise ValueError("concentrations must be finite")
    m_compose = dp.c.make_adaptive_composition(
        input_domain,
        input_metric,
        output_measure,
        (1.0, 1.0),
        list(concentrations),
    )

    def function(C):
        # Algorithm 1: http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5

        # 1.i Initialize C_1 = C inside compositor.
        #     C_i will be computed by `__make_np_sscp_projection`
        qbl = m_compose(C)
        del C  # only the compositor has the data now

        # 1.ii initialize P_1 with the identity matrix
        P = np.eye(input_desc.num_features)
        theta = np.zeros((0, input_desc.num_features))

        # computation of eigenvalues (1.iii) happens in a separate constructor

        # 2. only runs until concentrations are exhausted, not until num_features
        for concentration_i in concentrations:
            # 2.a.i: sample \hat{u}
            m_eigvec = (
                (input_domain, input_metric)
                >> _then_np_sscp_projection(P)  # P has orthonormal rows; see 2.b
                >> then_private_eigenvector(concentration_i)
                >> dp.as_array()
            )
            u = qbl(m_eigvec)

            # 2.a.ii: extend the set of eigenvectors
            theta = np.vstack((theta, P.T @ u))

            # 2.b: null_space returns an orthonormal basis, so P is a contraction.
            P = linalg.null_space(theta).T

        # the eigvec of a 1x1 matrix is always I, so it doesn't need to be released
        # so if down to the last eigvec, return the projection built up via postprocessing
        if P.shape[0] == 1:
            theta = np.vstack((theta, P))

        return theta.T[:, :num_components]

    return _make_measurement(
        input_domain,
        input_metric,
        m_compose.output_measure,
        function,
        # For each conditional query the bound is homogeneous in the score
        # range, so extend the unit-distance composition to arbitrary pairs.
        lambda d_in: _score_range(d_in) * m_compose.map((1.0, 1.0)),
    )


# generate then variant of the constructor
then_private_eigenvectors = to_then(make_private_eigenvectors)
