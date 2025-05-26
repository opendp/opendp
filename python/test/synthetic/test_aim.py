import opendp.prelude as dp

def test_make_ordinal_aim():
    from mbi import LinearMeasurement
    import numpy as np

    # Columns have cardinalities [3, 4, 5, 7] as specified in the domain
    np.random.seed(42)  # For reproducibility
    n_rows = 100
    data = np.zeros((n_rows, 4), dtype=int)
    
    # Generate data respecting the cardinalities for each column
    data[:, 0] = np.random.randint(0, 3, size=n_rows)  # Values 0-2
    data[:, 1] = np.random.randint(0, 11, size=n_rows)  # Values 0-3
    data[:, 2] = np.random.randint(0, 5, size=n_rows)  # Values 0-4
    data[:, 3] = np.random.randint(0, 7, size=n_rows)  # Values 0-6

    cardinalities=[3, 4, 5, 7]

    # Test the ordinal aim creation
    releases = [
        LinearMeasurement(
            noisy_measurement=np.bincount(data[:, i], minlength = cardinalities[i]), #maxine_edit: 
            clique=(i,),
        ) for i in range(4)
    ]

    m_aim = dp.synthetic.make_ordinal_aim(
        dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        d_in=1,
        d_out=0.5,
        releases=releases, 
        queries=[
            [0, 1, 2],
            [2, 3],
        ],
        weights=[1.0, 0.5]
    )

    m_aim(data)


dp.enable_features("contrib", "rust-stack-trace")
test_make_ordinal_aim()