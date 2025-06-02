import opendp.prelude as dp

def test_make_ordinal_aim():
    """
    Correctness test for make_ordinal_aim()
    """
    from mbi import LinearMeasurement
    import numpy as np

    # Columns have cardinalities [3, 4, 5, 7] as specified in the domain
    np.random.seed(42)  # For reproducibility
    n_rows = 100
    data = np.zeros((n_rows, 4), dtype=int)

    cardinalities=[3, 4, 5, 7]
    
    # Generate data respecting the cardinalities for each column
    data[:, 0] = np.random.randint(0, cardinalities[0], size=n_rows)  # Values 0-2
    data[:, 1] = np.random.randint(0, cardinalities[1], size=n_rows)  # Values 0-3
    data[:, 2] = np.random.randint(0, cardinalities[2], size=n_rows)  # Values 0-4
    data[:, 3] = np.random.randint(0, cardinalities[3], size=n_rows)  # Values 0-6

    # Test the ordinal aim creation
    # releases is how many times each integer appears in a given column; it thus has one array for each element of cardinalities
    releases = [
        LinearMeasurement(
            noisy_measurement=np.bincount(data[:, i], minlength = cardinalities[i]), #maxine_edit: minlength adjusted for each cardinality
            clique=(i,),
        ) for i in range(len(cardinalities))
    ]

    # cover line 69 domain type error
    try:
        domain = dp.numpy.arrayd_domain(shape=tuple(cardinalities), T=int)
        m_aim_wrong = dp.synthetic.make_ordinal_aim(
            domain,
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            d_in=1,
            d_out=0.5,
            releases=releases,
            queries=[
                [0, 1, 2],
                [2, 3]
            ],
            weights=[1.0, 0.5]
        )
    except ValueError as e:
        print(f"value error: {e}")
        
    
    # cover line 73 input cardinalities none error
    try: 
        m_aim_wrong = dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=None, T=int),
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
    except ValueError as e:
        print(f"value error: {e}")

    # cover line 76 cardinalities empty error
    try: 
        m_aim_wrong = dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=[], T=int),
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
    except ValueError as e:
        print(f"value error: {e}")
    
    # cover line 79 input metric not symmetric distance
    try: 
        m_aim_wrong = dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
            dp.l1_distance(T="f64"),
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
    except ValueError as e:
        print(f"value error: {e}")

    # cover line 82 output measure not zero conc divergence
    try: 
        m_aim_wrong = dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
            dp.symmetric_distance(),
            dp.renyi_divergence(),
            d_in=1,
            d_out=0.5,
            releases=releases, 
            queries=[
                [0, 1, 2],
                [2, 3],
            ],
            weights=[1.0, 0.5]
        )
    except ValueError as e:
        print(f"value error: {e}")

    # cover line 86 - 87 if weights are None or empty
    m_aim_weightless = dp.synthetic.make_ordinal_aim(
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
        weights=None
    )

    m_aim_weightless(data)
    print("done with run with weights = None")

    
    m_aim_weightempty = dp.synthetic.make_ordinal_aim(
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
        weights=[]
    )

    m_aim_weightempty(data)
    print("done with run with empty weights")

    # cover line 90 negative weight error
    try:
        m_aim_negative = dp.synthetic.make_ordinal_aim(
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
            weights=[-1.0, 0.5]
        )
    except ValueError as e:
        print(f"value error: {e}")

    # cover line 93 zero weight warning + remaining lines
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
            [0, 3],
            [1, 2]
        ],
        weights=[0, 0.3, 0.7, 1]
    )

    m_aim(data)
    print("done with run with empty weights + rest of normal queries")

dp.enable_features("contrib", "rust-stack-trace")