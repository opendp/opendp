from opendp._extrinsics.domains import _np_SSCP_domain
from opendp._extrinsics._utilities import to_then
from opendp.mod import Domain, Metric, Transformation, Measurement
from typing import List

# planning to make this public, but may make more API changes


def make_private_np_eigenvector(
    input_domain: Domain, input_metric: Metric, unit_epsilon: float
) -> Measurement:
    """Construct a Measurement that releases a private eigenvector from a covariance matrix.

    :param input_domain: instance of `_np_SSCP_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    """
    import numpy as np  # type: ignore[import]
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")

    d = input_domain.num_features

    def function(C):
        # Algorithm 2 Top Eigenvector Sampler
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=15
        # (1)
        A = unit_epsilon / 4 * (np.linalg.eigvalsh(C).max() * np.eye(d) - C)

        # (2)
        A_eigvals = np.linalg.eigvalsh(A)

        # Differs from the Amin et al.
        #   In 3.6 of https://eprints.whiterose.ac.uk/123206/7/simbingham8.pdf,
        #   the equality is against 1 not 0
        # Instead of using bounds of (1, d), increase the upper bound for numerical stability
        b = dp.binary_search(
            lambda b: sum(1 / (b + 2 * A_eigvals)) >= 1, bounds=(.9, float(d))
        )
        Omega = np.eye(d) + 2 * A / b

        # (3)
        M = np.exp(-(d - b) / 2) * (d / b) ** (d / 2)

        # prepare sampling
        Omega_inv = np.linalg.inv(Omega)

        while True:
            z = np.random.multivariate_normal(mean=np.zeros(d), cov=Omega_inv)
            u = z / np.linalg.norm(z)
            if np.exp(-u.T @ A @ u) / (M * (u.T @ Omega @ u) ** (d / 2)):
                return u

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(T=float),
        function,
        lambda d_in: d_in // 2 * unit_epsilon,
        TO=dp.Vec[input_domain.T],
    )


# generate then variant of the constructor
then_private_eigenvector = to_then(make_private_np_eigenvector)


def _make_np_SSCP_projection(
    input_domain: Domain, input_metric: Metric, P
) -> Transformation:
    """Construct a Transformation that projects an SSCP matrix.

    In order for the output_domain to follow,
    the singular values of `P` must be bounded above by 1, 
    so as not to increase the row norms in the implied X matrix,
    as the row norms are simply passed through.

    :param input_domain: instance of `_np_SSCP_domain(size=_, num_columns=_)`
    :param input_metric: instance of `symmetric_distance()`
    :param P: a projection whose singular values are no greater than 1
    """
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")
    if input_domain.num_features != P.shape[1]:
        raise ValueError(
            f"projection P (axis-1 size: {P.shape[1]}) does not conform with data in input_domain (num_features: {input_domain.num_features})"
        )

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        _np_SSCP_domain(
            **{**input_domain.descriptor._asdict(), "num_features": P.shape[0]}
        ),
        input_metric,
        # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5
        # Algorithm 1 step 2.c
        lambda cov: P @ cov @ P.T,  # c: update the covariance matrix
        lambda d_in: d_in,
    )


def make_private_np_eigenvectors(
    input_domain: Domain, input_metric: Metric, unit_epsilons: List[float]
) -> Measurement:
    import numpy as np  # type: ignore[import]
    from scipy.linalg import null_space  # type: ignore[import]
    import opendp.prelude as dp

    dp.assert_features("contrib", "floating-point")

    privacy_measure = dp.max_divergence(T=input_domain.T)

    if len(unit_epsilons) > input_domain.num_features - 1:
        raise ValueError(
            f"must specify at most {input_domain.num_features - 1} unit_epsilons"
        )

    m_compose = dp.c.make_sequential_composition(
        input_domain, input_metric, privacy_measure, 2, unit_epsilons
    )

    def function(C):
        nonlocal input_domain, input_metric
        # Algorithm 1: http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5

        # 1.i Initialize C_1 = C inside compositor. 
        #     C_i will be computed by `_make_np_SSCP_projection`
        qbl = m_compose(C)
        del C # only the compositor has the data now

        # 1.ii initialize P_1 with the identity matrix
        P = np.eye(input_domain.num_features)
        theta = np.zeros((0, input_domain.num_features))

        # computation of eigenvalues (1.iii) happens in a separate constructor

        # 2. only runs until epsilons are exhausted, not until num_features
        for epsilon_i in unit_epsilons:

            # 2.a.i: sample \hat{u}
            m_eigvec = _make_np_SSCP_projection( # 2.c happens inside this transformation
                input_domain, input_metric, P
            ) >> then_private_eigenvector(epsilon_i)
            u = qbl(m_eigvec)

            # 2.a.ii: extend the set of eigenvectors
            theta = np.vstack((theta, P.T @ u))

            # 2.b: update the projection, P maintains singular values with magnitude <= 1
            P = null_space(theta).T

        # the eigvec of a 1x1 matrix is always I, so it doesn't need to be released
        # so if down to the last eigvec, return the projection built up via postprocessing
        if P.shape[0] == 1:
            theta = np.vstack((theta, P))

        return theta.T

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        lambda d_in: d_in / 2 * m_compose.map(2),
    )


# generate then variant of the constructor
then_private_np_eigenvectors = to_then(make_private_np_eigenvectors)
