from opendp.extras.numpy import _symmetric_matrix_domain
from opendp.extras._utilities import to_then
from opendp._lib import import_optional_dependency
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
    input_domain: Domain,
    input_metric: Metric,
    concentration: float,
) -> Measurement:
    """Construct a Measurement that releases a private eigenvector from a covariance matrix.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear distance on symmetric matrices
    :param concentration: coefficient of u.T @ C @ u in the Bingham density.
        If eta is the concentration times the spectral diameter bound,
        the pure-DP privacy map is eta.
    """
    np = import_optional_dependency("numpy")
    import opendp.prelude as dp

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")

    if input_metric != dp.nuclear_distance():
        raise ValueError("expected nuclear distance input metric")

    if not np.isfinite(concentration) or concentration < 0:
        raise ValueError("concentration must be finite and non-negative")

    def function(C):
        return _sample_bingham(C, concentration)

    return _make_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(),
        function,
        # The log-normalizer is constant in the output, so the privacy
        # loss has range width at most the range of the quadratic score.
        lambda d_in: concentration * d_in,
        TO="Vec<f64>",
    )


def _sample_bingham(C, concentration):
    """Numerical proposal construction; Rust certifies the actual distribution."""
    from opendp._data import sample_bingham

    np = import_optional_dependency("numpy")
    C = np.asarray(C, dtype=float)
    d = C.shape[0]
    factor, b, l = [], 1.0, 0.0
    try:
        with np.errstate(over="raise", invalid="raise", divide="raise"):
            eigenvalues, Q = np.linalg.eigh(C - C[0, 0] * np.eye(d))
            l = float(eigenvalues[-1])
            a = concentration * (l - eigenvalues)
            b = _optimal_b(a, d)
            factor = (Q / np.sqrt(b + 2 * a)).ravel().tolist()
    except (FloatingPointError, OverflowError, ValueError, np.linalg.LinAlgError):
        # Hints are optional. The native sampler has an exact fallback.
        pass
    return sample_bingham(C.ravel().tolist(), d, concentration, factor, b, l)


# generate then variant of the constructor
then_private_eigenvector = to_then(make_private_eigenvector)


def _make_np_sscp_projection(
    input_domain: Domain, input_metric: Metric, P
) -> Transformation:
    """Construct a Transformation that projects an SSCP matrix.

    Internal helper: the caller must supply a contraction (operator norm <= 1).
    In eigendecomposition, P consists of an orthonormal basis of the remaining
    subspace (or the identity). The nuclear norm contracts under this projection.
    This precondition is not checked with an SVD in the inner loop; it follows
    from construction under the idealized-numerics assumption.

    :param input_domain: instance of `_symmetric_matrix_domain(num_features=_)`
    :param input_metric: nuclear distance on symmetric matrices
    :param P: a linear map with singular values no greater than 1
    """
    import opendp.prelude as dp

    np = import_optional_dependency("numpy")
    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != dp.nuclear_distance():
        raise ValueError("expected nuclear distance input metric")

    input_desc = input_domain.descriptor
    P = np.asarray(P)
    if P.ndim != 2 or not np.isfinite(P).all():
        raise ValueError("P must be a finite 2d matrix")
    if P.dtype not in {np.dtype("float32"), np.dtype("float64")}:
        raise ValueError("P must have a real float32 or float64 dtype")
    # Copy the public projection and use f64 throughout eigenvector sampling.
    P = P.astype(np.float64, copy=True)

    if input_desc.num_features != P.shape[1]:
        raise ValueError(
            f"projection P (axis-1 size: {P.shape[1]}) does not conform with data in input_domain (num_features: {input_desc.num_features})"
        )  # pragma: no cover

    return _make_transformation(
        input_domain,
        input_metric,
        _symmetric_matrix_domain(num_features=P.shape[0], T=dp.f64),
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
    d_in: float,
    d_out: float,
    num_components: int,
) -> Measurement:
    """Release leading eigenvectors (columns) of a symmetric matrix.

    :param input_domain: symmetric matrix domain
    :param input_metric: nuclear distance
    :param d_in: maximum nuclear distance between adjacent matrices
    :param d_out: total epsilon allowance at d_in
    :param num_components: number of leading eigenvectors to release.
        The budget is split evenly among sampled vectors; the last vector of
        a full basis follows by postprocessing.
    """
    import opendp.prelude as dp
    from opendp.context import _normalize_compositor

    np = import_optional_dependency("numpy")
    linalg = import_optional_dependency("scipy.linalg")

    dp.assert_features("contrib", "idealized-numerics")

    if not str(input_domain).startswith("NPSymmetricMatrixDomain"):
        raise ValueError("input_domain must be NPSymmetricMatrixDomain")
    if input_metric != dp.nuclear_distance():
        raise ValueError("expected nuclear distance input metric")
    if not np.isfinite(d_in) or d_in < 0 or not np.isfinite(d_out) or d_out < 0:
        raise ValueError("d_in and d_out must be finite and nonnegative")

    dimension = input_domain.descriptor.num_features
    if (
        not isinstance(num_components, (int, np.integer))
        or not 1 <= num_components <= dimension
    ):
        raise ValueError(
            "num_components must be an integer between 1 and the matrix dimension"
        )
    num_samples = min(int(num_components), dimension - 1)
    measure = dp.max_divergence()
    concentration = 0.0
    if num_samples and d_out > 0 and d_in > 0:
        m_compose, d_mids, _ = _normalize_compositor(
            input_domain,
            (input_metric, d_in),
            (measure, d_out),
            split_evenly_over=num_samples,
        )
        budget = d_mids[0]
        concentration = budget / d_in
        if not np.isfinite(concentration):
            raise ValueError("concentration must be finite")
        if concentration * d_in > budget:
            concentration = float(np.nextafter(concentration, 0.0))
    else:
        m_compose = dp.c.make_adaptive_composition(
            input_domain, input_metric, measure, d_in, [0.0] * num_samples
        )

    def function(C):
        # Algorithm 1: http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5

        # 1.i Initialize C_1 = C inside compositor.
        #     C_i will be computed by `_make_np_sscp_projection`
        qbl = m_compose(C)
        del C  # only the compositor has the data now

        # 1.ii initialize P_1 with the identity matrix
        P = np.eye(dimension)
        theta = np.zeros((0, dimension))

        # computation of eigenvalues (1.iii) happens in a separate constructor

        # 2. sample only the requested non-deterministic vectors
        for _ in range(num_samples):
            # 2.a.i: sample \hat{u}
            m_eigvec = (
                (input_domain, input_metric)
                >> _then_np_sscp_projection(P)  # P has orthonormal rows; see 2.b
                >> then_private_eigenvector(concentration)
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
        m_compose.map,
    )


# generate then variant of the constructor
then_private_eigenvectors = to_then(make_private_eigenvectors)
