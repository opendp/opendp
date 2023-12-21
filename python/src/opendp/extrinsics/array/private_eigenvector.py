import opendp.prelude as dp
from opendp.extrinsics.register import register_measurement


def make_private_eigenvector(input_domain, input_metric, unit_epsilon):
    """Construct a new Measurement that releases a private eigenvector from a covariance matrix."""
    import numpy as np

    dp.assert_features("contrib", "floating-point")

    if input_metric != dp.symmetric_distance():
        raise ValueError("expected symmetric distance input metric")

    descriptor = input_domain.descriptor
    d = descriptor["num_features"]

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
        b = dp.binary_search(
            lambda b: sum(1 / (b + 2 * A_eigvals)) >= 1, bounds=(1.0, float(d))
        )
        Omega = np.eye(d) + 2 * A / b

        # (3)
        M = np.exp(-(d - b) / 2) * (d / b) ** (d / 2)

        # prepare sampling
        Omega_inv = np.linalg.inv(Omega)

        while True:
            u = np.random.multivariate_normal(mean=np.zeros(d), cov=Omega_inv)
            u /= np.linalg.norm(u)
            if np.exp(-u.T @ A @ u) / ((M * u.T @ Omega @ u) ** (d / 2)):
                return u

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        dp.max_divergence(T=float),
        function,
        lambda d_in: d_in // 2 * unit_epsilon,
        TO=dp.Vec[descriptor["T"]],
    )


then_private_eigenvector = register_measurement(make_private_eigenvector)


def _make_np_cov_projection(input_domain, input_metric, P):
    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.np_cov_domain(**{**input_domain.descriptor, "num_features": P.shape[0]}),
        input_metric,
        lambda cov: P @ cov @ P.T,  # (c)
        lambda d_in: d_in,
    )


def make_private_eigenvectors(input_domain, input_metric, unit_epsilons):
    import numpy as np
    from scipy.linalg import null_space

    descriptor = input_domain.descriptor
    privacy_measure = dp.max_divergence(T=descriptor["T"])

    if len(unit_epsilons) > descriptor["num_features"] - 1:
        raise ValueError(f"must specify at most {descriptor['num_features'] - 1} unit_epsilons")

    m_compose = dp.c.make_sequential_composition(
        input_domain, input_metric, privacy_measure, 2, unit_epsilons
    )

    def function(cov):
        nonlocal input_domain, input_metric
        qbl = m_compose(cov)

        P = np.eye(len(cov))
        theta = np.zeros((0, cov.shape[1]))

        for epsilon_i in unit_epsilons:
            # c. update the covariance matrix
            m_eigvec = _make_np_cov_projection(input_domain, input_metric, P) >> then_private_eigenvector(epsilon_i)
            u = qbl(m_eigvec)

            # http://amin.kareemx.com/pubs/DPCovarianceEstimation.pdf#page=5
            # Algorithm 1
            # a. extend the set of eigenvectors
            theta = np.vstack((theta, P.T @ u))

            # b. update the projection
            P = null_space(theta).T
        
        if P.shape[0] == 1:
            theta = np.vstack((theta, P))

        return theta.T

    return dp.m.make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        lambda d_in: d_in // 2 * m_compose.map(2),
        TO="ExtrinsicObject",
    )


then_private_eigenvectors = register_measurement(make_private_eigenvectors)
