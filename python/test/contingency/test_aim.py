from opendp.extras.contingency import AIM, Count
import opendp.prelude as dp
import pytest
import re


@pytest.mark.parametrize("constructor", (dp.contingency._aim.make_aim_marginals, dp.contingency._mst.make_mst_marginals, dp.contingency._fixed.make_fixed_marginals))
def test_contingency_err_input_metric(constructor):
    with pytest.raises(ValueError, match=re.escape("input_metric (DiscreteDistance()) must be frame_distance")):
        constructor(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            dp.discrete_distance(),
            dp.zero_concentrated_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
        )


@pytest.mark.parametrize("constructor", (dp.contingency._aim.make_aim_marginals, dp.contingency._mst.make_mst_marginals, dp.contingency._fixed.make_fixed_marginals))
def test_contingency_err_output_measure(constructor):
    with pytest.raises(
        ValueError, match="output_measure must be zero_concentrated_divergence"
    ):
        constructor(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            dp.frame_distance(dp.symmetric_distance()),
            dp.renyi_divergence(),
            d_in=1,
            d_out=0.5,
            releases=[],
        )


def test_make_ordinal_aim_err_weights():
    with pytest.raises(ValueError, match="weight must be positive"):
        Count(("A", "B", "C"), weight=-1.0)


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

    m_aim = dp.contingency.make_aim(
        dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        d_in=1,
        d_out=0.5,
        releases=releases,
        queries=[
            Count([0, 1, 2]),
            Count([2, 3]),
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

    m_aim = dp.contingency.make_aim(
        dp.numpy.array2_domain(cardinalities=cardinalities, T=int),
        dp.symmetric_distance(),
        dp.zero_concentrated_divergence(),
        d_in=1,
        d_out=0.5,
        releases=releases,
        queries=[],
    )

    m_aim(data)
