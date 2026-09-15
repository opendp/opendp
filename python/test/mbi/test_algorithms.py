from opendp.extras.mbi import (
    Fixed,
    mirror_descent,
    Count,
    AIM,
    MST,
    Marginals,
    Sequential,
)
import opendp.prelude as dp
import pytest
import re

from ..helpers import ids


@pytest.mark.parametrize(
    "algorithm",
    (
        dp.mbi.AIM(),
        dp.mbi.MST(),
        dp.mbi.Sequential(
            algorithms=[
                Fixed(queries=[Count(("A",))]),
                AIM(),
            ],
            weights=[2, 8],
        ),
        dp.mbi.Fixed(queries=[Count(("A",))]),
    ),
    ids=ids,
)
def test_algorithm_err_elements(algorithm):
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]

    model = mirror_descent(mbi.Domain(("A",), (2,)), [])

    msg = "input_domain columns must be bounded"
    with pytest.raises(ValueError, match=re.escape(msg)):
        algorithm.make_marginals(
            dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=int))]),
            dp.frame_distance(dp.symmetric_distance()),
            dp.zero_concentrated_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals=Marginals(),
            model=model,
        )

    msg = "input_metric (DiscreteDistance()) must be frame_distance"
    with pytest.raises(ValueError, match=re.escape(msg)):
        algorithm.make_marginals(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(bounds=(0, 10)))]
            ),
            dp.discrete_distance(),
            dp.zero_concentrated_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals=Marginals(),
            model=model,
        )

    msg = "output_measure (RenyiDivergence) must be max_divergence() or zero_concentrated_divergence()"
    with pytest.raises(ValueError, match=re.escape(msg)):
        algorithm.make_marginals(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(bounds=(0, 10)))]
            ),
            dp.frame_distance(dp.symmetric_distance()),
            dp.renyi_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=0.5,
            marginals=Marginals(),
            model=model,
        )

    with pytest.raises(ValueError, match="model must be a MarkovRandomField"):
        algorithm.make_marginals(
            dp.lazyframe_domain(
                [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
            ),
            dp.frame_distance(dp.symmetric_distance()),
            dp.max_divergence(),
            d_in=[dp.polars.Bound(per_group=1)],
            d_out=1.0,
            marginals=Marginals(),
            # Fixed can intentionally start without a model because its fixed
            # workload provides the first model constraint. Pass an invalid
            # non-None value so every algorithm still exercises validation.
            model=object(),
        )


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(queries=-1), "queries (-1) must be positive"),
        (dict(queries=[]), "queries must not be empty"),
        (dict(measure_split=2), "measure_split (2) must be in (0, 1]"),
        (dict(max_size=-1), "max_size (-1) must be positive"),
    ],
    ids=ids,
)
def test_aim_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        AIM(**kwargs)


def test_aim_scores_use_l1_sensitivity_with_structured_bounds():
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]
    from opendp.extras.mbi._aim import _make_aim_scores
    from opendp.extras.mbi._utilities import typed_dict_distance, typed_dict_domain

    query = Count(("A",))
    model = mirror_descent(mbi.Domain(("A",), (2,)), [])
    input_domain = typed_dict_domain(
        {("A",): dp.numpy.arrayd_domain(shape=(2,), T="i32")}
    )
    input_metric = typed_dict_distance(
        dp.l01inf_distance(dp.absolute_distance(T="i32"))
    )

    scores = _make_aim_scores(
        input_domain, input_metric, [query], expectations=[0.0], model=model
    )
    distance = {("A",): (2, 6, 3)}

    # The old zCDP path selected using min(6, sqrt(2) * 3), which is too
    # small for the published L1 score. Selection must use Delta_1 = 6.
    assert scores.map(distance) == 6.0
    assert scores.map(distance) > 3 * 2**0.5


def test_aim_penalty_uses_measurement_noise(monkeypatch):
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]
    import opendp.extras.mbi._aim as aim
    from opendp.extras.mbi._utilities import typed_dict_distance, typed_dict_domain

    queries = [Count(("A",)), Count(("B",))]
    model = mirror_descent(mbi.Domain(("A", "B"), (2, 2)), [])
    input_domain = typed_dict_domain(
        {
            ("A",): dp.numpy.arrayd_domain(shape=(2,), T="i32"),
            ("B",): dp.numpy.arrayd_domain(shape=(2,), T="i32"),
        }
    )
    input_metric = typed_dict_distance(
        dp.l01inf_distance(dp.absolute_distance(T="i32"))
    )
    captured_expectations = []
    original_scores = aim._make_aim_scores

    def spy_scores(input_domain, input_metric, queries, expectations, model):
        captured_expectations.append(expectations)
        return original_scores(input_domain, input_metric, queries, expectations, model)

    measurement_scales = iter([7.0, 11.0])

    def fake_measurement_scale(make, *, d_in, d_out, T):
        assert d_out == 0.9
        return next(measurement_scales)

    # Give noisy-max an intentionally different selection scale. Each penalty
    # must retain its candidate's measurement scale, not use the selection scale.
    monkeypatch.setattr(aim, "_make_aim_scores", spy_scores)
    monkeypatch.setattr(aim, "binary_search_param", fake_measurement_scale)
    monkeypatch.setattr(aim, "binary_search_chain", lambda make, **kwargs: make(0.25))

    selection = aim._make_aim_select(
        input_domain,
        input_metric,
        dp.max_divergence(),
        d_in={("A",): (2, 6, 3), ("B",): (1, 2, 2)},
        d_out=0.1,
        queries=queries,
        model=model,
        max_size=float("inf"),
        d_measure=0.9,
    )

    assert selection is not None
    assert captured_expectations == [[7.0, 11.0]]


def test_aim_exhaustion():
    # tests how algorithm behaves when all workload queries are ineligible for selection
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    m_aim = dp.mbi.AIM(max_size=1e-10).make_marginals(
        dp.lazyframe_domain(
            [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
        ),
        dp.frame_distance(dp.symmetric_distance()),
        dp.max_divergence(),
        d_in=[dp.polars.Bound(per_group=1)],
        d_out=1.0,
        marginals=Marginals(),
        model=dp.mbi.mirror_descent(mbi.Domain(("A",), (2,)), []),
    )

    m_aim(pl.LazyFrame({"A": [0]}))


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(queries=[]), "queries must have at least one element"),
        (dict(queries=[2]), "queries must be of type Count"),
    ],
    ids=ids,
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
    ids=ids,
)
def test_mst_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        MST(**kwargs)


@pytest.mark.parametrize(
    "kwargs,message",
    [
        (dict(algorithms=[]), "algorithms must contain at least one element"),
        (
            dict(algorithms=[False]),
            "algorithms ([False]) must be instances of Algorithm",
        ),
        (dict(algorithms=[MST()], weights=[]), "algorithms and weights must contain"),
        (dict(algorithms=[MST()], weights=[0]), "weights ([0]) must be positive"),
    ],
    ids=ids,
)
def test_sequential_init(kwargs, message):
    with pytest.raises(ValueError, match=re.escape(message)):
        Sequential(**kwargs)
