from opendp.extras.numpy import _sscp_domain
from opendp.extras._utilities import to_then
from opendp._lib import get_np_csprng, import_optional_dependency
from opendp.mod import Domain, Metric, Transformation, Measurement
from opendp._internal import _make_measurement, _make_transformation

# planning to make this public, but may make more API changes


def make_private_eigenvector(
    input_domain: Domain, input_metric: Metric, unit_epsilon: float
) -> Measurement:
    """Construct a Measurement that releases a private eigenvector from a covariance matrix.

    :param input_domain: instance of `_sscp_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param unit_epsilon: ε-expenditure per changed record in the input data
    """
    np = import_optional_dependency('numpy')
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")
    
    np_csprng = get_np_csprng()
    input_desc = input_domain.descriptor

    if input_desc.p != 2:
        raise ValueError("input_domain must have bounded L2-norm")  # pragma: no cover

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")  # pragma: no cover

    d = input_desc.num_features

    def function(C):
        # Algorithm 2 Top Eigenvector Sampler
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=15
        # scale norm down to at most 1
        C = C / input_desc.norm**2

        # (1)
        A = unit_epsilon / 4 * (np.linalg.eigvalsh(C).max() * np.eye(d) - C)

        # (2)
        A_eigvals = np.linalg.eigvalsh(A)

        # This algorithm first finds parameters for an angular central gaussian distribution (ACG),
        #   which acts as an envelope distribution for rejection sampling.
        # The rejection rate will be minimized when b is chosen optimally.
        # b is chosen optimally when the ACG is least entropic, but is still an envelope.
        # Criteria for being an envelope are given in 3.4 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf

        # Differs from the Amin et al. in two ways:
        # 1. In 3.6 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf,
        #   the equality is against 1 not 0
        # 2. Instead of using bounds of (1, d), decrease the lower bound for numerical stability
        b = dp.binary_search(
            lambda b: sum(1 / (b + 2 * A_eigvals)) >= 1, bounds=(0.9, float(d))
        )
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
            if np.exp(-u.T @ A @ u) / (M * (u.T @ Omega @ u) ** (d / 2)):
                return u

    return _make_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(),
        function,
        lambda d_in: d_in / 2 * unit_epsilon,
        TO=dp.Vec[input_desc.T],
    )


# generate then variant of the constructor
then_private_eigenvector = to_then(make_private_eigenvector)


def make_np_sscp_projection(
    input_domain: Domain, input_metric: Metric, P
) -> Transformation:
    """Construct a Transformation that projects an SSCP matrix.

    In order for the output_domain to follow,
    the singular values of `P` must be bounded above by 1,
    so as not to increase the row norms in the implied X matrix,
    as the row norms are simply passed through.

    :param input_domain: instance of `_sscp_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param P: a projection whose singular values are no greater than 1
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")
    input_desc = input_domain.descriptor

    if input_desc.num_features != P.shape[1]:
        raise ValueError(
            f"projection P (axis-1 size: {P.shape[1]}) does not conform with data in input_domain (num_features: {input_domain.num_features})"
        )  # pragma: no cover

    kwargs = input_desc._asdict() | {"num_features": P.shape[0]}
    return _make_transformation(
        input_domain,
        input_metric,
        _sscp_domain(**kwargs),
        input_metric,
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5
        # Algorithm 1 step 2.c
        lambda cov: P @ cov @ P.T,  # c: update the covariance matrix
        lambda d_in: d_in,
    )


# generate then variant of the constructor
then_np_sscp_projection = to_then(make_np_sscp_projection)


def make_private_eigenvectors(
    input_domain: Domain, input_metric: Metric, unit_epsilons: list[float]
) -> Measurement:
    np = import_optional_dependency('numpy')
    import opendp.prelude as dp
    linalg = import_optional_dependency('scipy.linalg')

    dp.assert_features("contrib", "floating-point")

    input_desc = input_domain.descriptor
    if input_desc.p != 2:
        raise ValueError("input_domain must have bounded L2 row norm")  # pragma: no cover

    if len(unit_epsilons) > input_desc.num_features - 1:
        raise ValueError(
            f"must specify at most {input_desc.num_features - 1} unit_epsilons"
        )  # pragma: no cover

    privacy_measure = dp.max_divergence()
    m_compose = dp.c.make_sequential_composition(
        input_domain, input_metric, privacy_measure, 2, unit_epsilons
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
        for epsilon_i in unit_epsilons:
            # 2.a.i: sample \hat{u}
            m_eigvec = (
                (input_domain, input_metric)
                >> then_np_sscp_projection(P)  # 2.c happens inside this transformation
                >> then_private_eigenvector(epsilon_i)
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
        lambda d_in: d_in / 2 * m_compose.map(2),
    )


# generate then variant of the constructor
then_private_eigenvectors = to_then(make_private_eigenvectors)
