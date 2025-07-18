from opendp.extras.mbi import (
    Fixed,
    mirror_descent,
    Count,
    AIM,
    MST,
    Sequential,
)
import opendp.prelude as dp
import pytest
import re


def make_fixed_marginals(*args, **kwargs):
    return dp.mbi._fixed.make_fixed_marginals(
        *args, **kwargs, algorithm=Fixed(queries=[Count(("A",))])
    )


def make_sequential_marginals(*args, **kwargs):
    return dp.mbi._sequential.make_sequential_marginals(
        *args,
        **kwargs,
        algorithm=Sequential(
            algorithms=[
                Fixed(queries=[Count(("A",))]),
                AIM(),
            ],
            weights=[2, 8],
        )
    )


@pytest.mark.parametrize(
    "constructor",
    (
        dp.mbi._aim.make_aim_marginals,
        dp.mbi._mst.make_mst_marginals,
        make_fixed_marginals,
        make_sequential_marginals,
    ),
)
def test_algorithm_err_elements(constructor):
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped]

    model = mirror_descent(mbi.Domain(("A",), (2,)), [])

    msg = "input_domain columns must be bounded"
    with pytest.raises(ValueError, match=re.escape(msg)):
        constructor(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            dp.frame_distance(dp.symmetric_distance()),
            dp.zero_concentrated_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals={},
            model=model,
        )

    msg = "input_metric (DiscreteDistance()) must be frame_distance"
    with pytest.raises(ValueError, match=re.escape(msg)):
        constructor(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(bounds=(0, 10)))]
            ),
            dp.discrete_distance(),
            dp.zero_concentrated_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals={},
            model=model,
        )

    msg = "output_measure (RenyiDivergence) must be max_divergence() or zero_concentrated_divergence()"
    with pytest.raises(ValueError, match=re.escape(msg)):
        constructor(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(bounds=(0, 10)))]
            ),
            dp.frame_distance(dp.symmetric_distance()),
            dp.renyi_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals={},
            model=model,
        )

    with pytest.raises(ValueError, match="model must be a MarkovRandomField"):
        constructor(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
            ),
            dp.frame_distance(dp.symmetric_distance()),
            dp.max_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=1.0,
            marginals={},
            model=None,
        )


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(queries=-1), "queries (-1) must be positive"),
        (dict(queries=[]), "queries must not be non-empty"),
        (dict(measure_split=2), "measure_split (2) must be in (0, 1]"),
        (dict(max_size=-1), "max_size (-1) must be positive"),
    ],
)
def test_aim_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        AIM(**kwargs)


def test_aim_exhaustion():
    pytest.importorskip("mbi")
    import mbi
    import polars as pl

    m_aim = dp.mbi._aim.make_aim_marginals(
        dp.lazyframe_domain(
            [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
        ),
        dp.frame_distance(dp.symmetric_distance()),
        dp.max_divergence(),
        d_in=[dp.polars.Bound(per_group=1)],
        d_out=1.0,
        marginals={},
        model=dp.mbi.mirror_descent(mbi.Domain(("A",), (2,)), []),
        algorithm=AIM(max_size=1e-10),
    )

    m_aim(pl.LazyFrame({"A": [0]}))


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(queries=[]), "queries must have at least one element"),
        (dict(queries=[2]), "queries must be of type Count"),
    ],
)
def test_fixed_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        Fixed(**kwargs)


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(measure_split=2), "measure_split (2) must be in (0, 1]"),
        (dict(num_selections=0), "num_selections (0) must be positive"),
    ],
)
def test_mst_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        MST(**kwargs)


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(algorithms=[]), "algorithms must contain at least one element"),
        (dict(algorithms=[False]), "algorithms must be instances of Algorithm"),
        (dict(algorithms=[MST()], weights=[]), "algorithms and weights must contain"),
        (dict(algorithms=[MST()], weights=[0]), "weights must be positive"),
    ],
)
def test_sequential_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        Sequential(**kwargs)
