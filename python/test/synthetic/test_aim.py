import opendp.prelude as dp
import pytest


@pytest.mark.parametrize(
    "domain, message",
    [
        (
            dp.numpy.arrayd_domain(shape=(1, 2, 3), T=int),
            "input_domain must be opendp.extras.numpy.array2_domain",
        ),
        (
            dp.numpy.array2_domain(cardinalities=None, T=int),
            "input_domain cardinalities must be defined",
        ),
    ],
)
def test_make_ordinal_aim_err_input_domain(domain, message):
    with pytest.raises(ValueError, match=message):
        dp.synthetic.make_ordinal_aim(
            domain,
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
            queries=[[0, 1, 2], [2, 3]],
        )


def test_make_ordinal_aim_err_input_metric():
    with pytest.raises(ValueError, match="input_metric must be symmetric_distance"):
        dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=[1], T=int),
            dp.discrete_distance(),
            dp.zero_concentrated_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
            queries=[[0, 1, 2], [2, 3]],
        )


def test_make_ordinal_aim_err_output_measure():
    with pytest.raises(
        ValueError, match="output_measure must be zero_concentrated_divergence"
    ):
        dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=[1], T=int),
            dp.symmetric_distance(),
            dp.renyi_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
            queries=[[0, 1, 2], [2, 3]],
        )


@pytest.mark.parametrize("weights,message", [
    ([-1.0, 0.5], "weights must be non-negative"),
    ([1.0], "weights must have the same length as queries"),
])
def test_make_ordinal_aim_err_weights(weights, message):
    with pytest.raises(ValueError, match=message):
        dp.synthetic.make_ordinal_aim(
            dp.numpy.array2_domain(cardinalities=[1, 2, 3, 4], T=int),
            dp.symmetric_distance(),
            dp.zero_concentrated_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
            queries=[[0, 1, 2], [2, 3]],
            weights=weights,
        )


def test_make_ordinal_aim():
    mbi = pytest.importorskip("mbi")
    import numpy as np
    np.random.seed(42)  # For reproducibility
    n_rows = 100

    cardinalities = [3, 4, 5, 7]

    # Generate data respecting the cardinalities for each column
    data = np.column_stack(
        [np.random.randint(0, c, size=n_rows) for c in cardinalities]
    )

    # Initialize 1-D marginals
    # Releases is how many times each integer appears in a given column.
    # One array for each element of cardinalities.
    releases = [
        mbi.LinearMeasurement(
            noisy_measurement=np.bincount(data[:, i], minlength=c),
            clique=(i,),
        )
        for i, c in enumerate(cardinalities)
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
    )

    m_aim(data)


def test_make_ordinal_aim_no_queries():
    mbi = pytest.importorskip("mbi")
    import numpy as np
    np.random.seed(42)  # For reproducibility
    n_rows = 100

    cardinalities = [3, 4, 5, 7]

    # Generate data respecting the cardinalities for each column
    data = np.column_stack(
        [np.random.randint(0, c, size=n_rows) for c in cardinalities]
    )

    # Initialize 1-D marginals
    # Releases is how many times each integer appears in a given column.
    # One array for each element of cardinalities.
    releases = [
        mbi.LinearMeasurement(
            noisy_measurement=np.bincount(data[:, i], minlength=c),
            clique=(i,),
        )
        for i, c in enumerate(cardinalities)
    ]

    m_aim = dp.synthetic.make_ordinal_aim(
        dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        d_in=1,
        d_out=0.5,
        releases=releases,
        queries=[],
    )

    m_aim(data)
