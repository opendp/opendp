from opendp.extras.mbi import (
    Algorithm,
    Fixed,
    mirror_descent,
    Count,
    AIM,
    MST,
    Marginals,
    Sequential,
)
import math
import re

import opendp.prelude as dp
import pytest

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


def test_algorithm_hbc_gates(monkeypatch):
    import opendp.mod as mod
    from opendp.extras.mbi._utilities import _snapshot_algorithm

    class CustomAlgorithm(Algorithm):
        def make_marginals(self, *args, **kwargs):
            raise NotImplementedError

    class CustomFixed(Fixed):
        pass

    fixed = Fixed(queries=[Count(("A",))])
    overridden = Fixed(queries=[Count(("A",))])
    overridden.make_marginals = lambda *args, **kwargs: None  # type: ignore[method-assign]
    overridden_postinit = Fixed(queries=[Count(("A",))])
    overridden_postinit.__post_init__ = lambda: None  # type: ignore[method-assign]

    custom_estimator = Fixed(
        queries=[Count(("A",))],
        estimator=lambda *args, **kwargs: None,
    )
    nested = Sequential(
        algorithms=[
            AIM(),
            Sequential(algorithms=[CustomFixed(queries=[Count(("A",))])]),
        ]
    )
    nested_estimator = Sequential(algorithms=[custom_estimator])
    trusted = [AIM(), MST(), fixed, Sequential(algorithms=[fixed, AIM()])]
    untrusted = [
        CustomAlgorithm(),
        CustomFixed(queries=[Count(("A",))]),
        overridden,
        overridden_postinit,
        custom_estimator,
        nested,
        nested_estimator,
    ]

    with monkeypatch.context() as patch:
        patch.setattr(mod, "GLOBAL_FEATURES", mod.GLOBAL_FEATURES - {"honest-but-curious"})
        for algorithm in trusted:
            _snapshot_algorithm(algorithm)
        for algorithm in untrusted:
            with pytest.raises(dp.OpenDPException, match="honest-but-curious"):
                _snapshot_algorithm(algorithm)

    for algorithm in untrusted:
        _snapshot_algorithm(algorithm)


def test_count_normalizes_user_values():
    class Weight:
        def __float__(self):
            return 2.0

        def __mul__(self, other):
            raise AssertionError("custom weight executed on private data")

    query = Count(["A"], weight=Weight())  # type: ignore[arg-type]
    assert type(query.weight) is float
    assert query.weight == 2.0

    class CustomCount(Count):
        pass

    for algorithm in (
        AIM(queries=[CustomCount(("A",))]),
        Fixed(queries=[CustomCount(("A",))]),
    ):
        assert type(algorithm.queries[0]) is Count

    with pytest.raises(ValueError, match="by must contain column names"):
        Count([1])  # type: ignore[list-item]
    for weight in [float("nan"), math.inf, 0.0, -1.0]:
        with pytest.raises(ValueError, match="finite and positive"):
            Count(("A",), weight=weight)


def test_algorithm_configuration_snapshots_lists():
    count = Count(("A",))
    fixed_queries = [count]
    aim_queries = [count]
    fixed = Fixed(queries=fixed_queries)
    aim = AIM(queries=aim_queries)
    algorithms = [fixed, aim]
    weights = [1.0, 2.0]
    sequential = Sequential(algorithms=algorithms, weights=weights)

    fixed_queries.clear()
    aim_queries.clear()
    algorithms.clear()
    weights[0] = 10.0

    assert fixed.queries == (count,)
    assert aim.queries == (count,)
    assert sequential.algorithms == (fixed, aim)
    assert sequential.weights == (1.0, 2.0)


def test_algorithm_configuration_can_change_before_capture():
    from opendp.extras.mbi._utilities import _snapshot_algorithm

    count = Count(("A",), weight=1.0)
    first = Fixed(queries=[count])
    nested = Sequential(
        algorithms=[first, AIM(queries=[Count(("A",))])],
        weights=[1.0, 2.0],
    )
    captured = _snapshot_algorithm(nested)

    count.weight = 3.0
    first.queries[0].weight = 4.0
    first.queries = [Count(("B",))]
    nested.algorithms = [MST()]
    nested.weights = [9.0]

    captured_fixed = captured.algorithms[0]
    assert isinstance(captured_fixed, Fixed)
    assert captured_fixed.queries[0].weight == 1.0
    assert captured_fixed.queries[0].by == ("A",)
    assert captured.weights == (1.0, 2.0)
    assert len(captured.algorithms) == 2


def test_fixed_direct_make_marginals_snapshots_configuration():
    pytest.importorskip("mbi")
    import polars as pl  # type: ignore[import-not-found]

    algorithm = Fixed(queries=[Count(("A",))])
    measurement = algorithm.make_marginals(
        input_domain=dp.lazyframe_domain(
            [dp.series_domain("A", dp.atom_domain(T="i32", bounds=(0, 1)))]
        ),
        input_metric=dp.frame_distance(dp.symmetric_distance()),
        output_measure=dp.max_divergence(),
        d_in=[dp.polars.Bound(per_group=1)],
        d_out=5.0,
        marginals=Marginals(),
        model=None,
    )

    def changed_estimator(*args, **kwargs):
        raise AssertionError("configuration mutated after construction")

    algorithm.estimator = changed_estimator
    algorithm.queries = [Count(("B",))]
    _, model = measurement(pl.LazyFrame({"A": pl.Series("A", [0, 1], dtype=pl.Int32)}))
    assert model is not None


def test_aim_rejects_mutable_query_objects():
    class QueryLike:
        by = ["A"]
        weight = 1.0

    with pytest.raises(ValueError, match="queries must be of type Count"):
        AIM(queries=[QueryLike()])  # type: ignore[list-item]


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


def test_mst_and_aim_scores_use_l1_sensitivity():
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]
    from opendp.extras.mbi._aim import _make_aim_scores
    from opendp.extras.mbi._mst import _make_mst_scores
    from opendp.extras.mbi._utilities import typed_dict_distance, typed_dict_domain

    clique = ("A", "B")
    domain = typed_dict_domain({clique: dp.numpy.arrayd_domain(shape=(9, 9), T="i32")})
    metric = typed_dict_distance(dp.l01inf_distance(dp.absolute_distance(T="i32")))
    model = mirror_descent(mbi.Domain(clique, (9, 9)), [])
    distance = {clique: (9, 9, 1)}

    edge = ["A", "B"]
    mst = _make_mst_scores(domain, metric, [edge], model)  # type: ignore[list-item]
    aim = _make_aim_scores(domain, metric, [Count(clique)], {clique: 0.0}, model)
    assert mst.map(distance) == 9.0
    assert aim.map(distance) == 9.0

    import numpy as np  # type: ignore[import-not-found]

    exact = {clique: np.zeros((9, 9), dtype=np.int32)}
    original_score = mst(exact)
    edge[0] = "B"
    assert mst(exact) == original_score


def test_aim_scores_snapshot_workload():
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]
    import numpy as np  # type: ignore[import-not-found]
    from opendp.extras.mbi._aim import _make_aim_scores
    from opendp.extras.mbi._utilities import typed_dict_distance, typed_dict_domain

    clique: tuple[str, ...] = ("A",)
    queries = [Count(clique, weight=2.0)]
    expectations = {clique: 1.0}
    transformation = _make_aim_scores(
        typed_dict_domain({clique: dp.numpy.arrayd_domain(shape=(2,), T="i32")}),
        typed_dict_distance(dp.l01inf_distance(dp.absolute_distance(T="i32"))),
        queries,
        expectations,
        mirror_descent(mbi.Domain(clique, (2,)), []),
    )
    data = {clique: np.array([1, 0], dtype=np.int32)}
    distance = {clique: (1, 1, 1)}
    expected_score = transformation(data)
    expected_map = transformation.map(distance)

    queries[0].weight = 100.0
    queries[0].by = ("B",)
    queries.clear()
    expectations[clique] = 1_000.0
    assert transformation(data) == pytest.approx(expected_score)
    assert transformation.map(distance) == expected_map


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


def test_aim_selects_correlated_pairs():
    pytest.importorskip("mbi")
    import numpy as np  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]

    # eight columns in four perfectly correlated pairs: B{i} == A{i}
    n = 20_000
    rng = np.random.default_rng(1)
    columns = [f"{letter}{i}" for i in range(4) for letter in "AB"]
    data = {}
    for i in range(4):
        data[f"A{i}"] = rng.integers(0, 4, n)
        data[f"B{i}"] = data[f"A{i}"].copy()
    df = pl.DataFrame({c: data[c] for c in columns})

    context = dp.Context.compositor(
        data=df.lazy(),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=0.002),
    )
    table = (
        context.query(rho=0.002)
        .select(*columns)
        .contingency_table(
            keys={c: [0, 1, 2, 3] for c in columns},
            algorithm=dp.mbi.AIM(queries=2),
        )
        .release()
    )

    selected = {frozenset(clique) for clique in table.marginals.cliques()}
    found = sum(frozenset((f"A{i}", f"B{i}")) in selected for i in range(4))
    assert found >= 3, f"only {found}/4 correlated pairs selected"

    # mean total variation distance over all two-way marginals;
    # the independence model scores ~0.107 here
    synthetic = table.synthesize().drop_nulls()
    errors = []
    for i, left in enumerate(columns):
        for right in columns[i + 1 :]:
            p = np.bincount(
                df[left].to_numpy() * 4 + df[right].to_numpy(), minlength=16
            ) / len(df)
            q = np.bincount(
                synthetic[left].to_numpy() * 4 + synthetic[right].to_numpy(),
                minlength=16,
            ) / len(synthetic)
            errors.append(0.5 * np.abs(p - q).sum())
    assert np.mean(errors) < 0.04


@pytest.mark.parametrize(
    "output_measure, ratio",
    [(dp.max_divergence(), 2.0), (dp.zero_concentrated_divergence(), 2**1.5)],
)
def test_aim_penalty_is_per_clique(monkeypatch, output_measure, ratio):
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-not-found]
    import polars as pl  # type: ignore[import-not-found]
    from opendp.extras.mbi import _aim
    from opendp.extras.mbi._aim import _expand_queries, _make_aim_select
    from opendp.extras.mbi._utilities import make_stable_marginals

    penalties = []
    inner = _aim._make_aim_scores

    def spy(input_domain, input_metric, queries, expectations, model):
        # test verifies the penalty is not shared by every candidate
        penalties.append(
            expectations
            if isinstance(expectations, dict)
            else {q.by: expectations for q in queries}
        )
        return inner(input_domain, input_metric, queries, expectations, model)

    monkeypatch.setattr(_aim, "_make_aim_scores", spy)

    columns = ("A", "B", "C")
    # an individual contributes up to four records, but only one per group of A,
    # tightening the sensitivity of the ("A",) marginal
    d_in = [
        dp.polars.Bound(per_group=4),
        dp.polars.Bound(by=[pl.col("A")], per_group=1, num_groups=2),
    ]
    queries = _expand_queries(2, list(columns))

    t_marginals = make_stable_marginals(
        dp.lazyframe_domain(
            [
                dp.series_domain(c, dp.atom_domain(T="u32", bounds=(0, 3)))
                for c in columns
            ]
        ),
        dp.frame_distance(dp.symmetric_distance()),
        [q.by for q in queries],
    )

    _make_aim_select(
        *t_marginals.output_space,
        output_measure,
        d_in=t_marginals.map(d_in),
        d_out=0.1,
        d_measure=0.9,
        queries=queries,
        model=mirror_descent(mbi.Domain(columns, (4, 4, 4)), []),
        max_size=80.0,
    )

    assert penalties, "no SELECT step ran"
    for expectations in penalties:
        assert expectations[("B",)] == pytest.approx(ratio * expectations[("A",)])


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


def test_mst_selection_budget_respects_conservative_composition():
    # Repeated rounded-up privacy losses can exceed a naively divided budget.
    pytest.importorskip("mbi")
    import itertools
    import mbi  # type: ignore[import-untyped,import-not-found]

    from opendp.extras.mbi._mst import _make_mst_select
    from opendp.extras.mbi._utilities import make_stable_marginals

    columns = tuple(f"C{i}" for i in range(9))
    edges = list(itertools.combinations(columns, 2))
    domain = dp.lazyframe_domain(
        [dp.series_domain(c, dp.atom_domain(T="u32", bounds=(0, 1))) for c in columns]
    )
    metric = dp.frame_distance(dp.symmetric_distance())
    d_in = [dp.polars.Bound(per_group=1)]
    t_marginals = make_stable_marginals(domain, metric, edges)
    d_marginals = t_marginals.map(d_in)
    budget = 0.01
    m_select = _make_mst_select(
        *t_marginals.output_space,
        output_measure=dp.zero_concentrated_divergence(),
        d_in=d_marginals,
        d_out=budget,
        edges=edges,
        model=mirror_descent(mbi.Domain(columns, (2,) * len(columns)), []),
    )

    assert m_select.check(d_marginals, budget)


def test_aim_selection_requires_positive_budget():
    # An exhausted selection allocation must not search negative noise scales.
    pytest.importorskip("mbi")
    import mbi  # type: ignore[import-untyped,import-not-found]

    from opendp.extras.mbi._aim import _make_aim_select
    from opendp.extras.mbi._utilities import make_stable_marginals

    columns = ("A", "B")
    queries = [Count(("A",)), Count(("B",))]
    domain = dp.lazyframe_domain(
        [dp.series_domain(c, dp.atom_domain(T="u32", bounds=(0, 1))) for c in columns]
    )
    metric = dp.frame_distance(dp.symmetric_distance())
    t_marginals = make_stable_marginals(domain, metric, [q.by for q in queries])
    d_in = t_marginals.map([dp.polars.Bound(per_group=1)])
    model = mirror_descent(mbi.Domain(columns, (2, 2)), [])

    assert (
        _make_aim_select(
            *t_marginals.output_space,
            output_measure=dp.zero_concentrated_divergence(),
            d_in=d_in,
            d_out=0.0,
            d_measure=1.0,
            queries=queries,
            model=model,
            max_size=80.0,
        )
        is None
    )


def test_fixed_noise_scale_search_is_nonnegative(monkeypatch):
    pytest.importorskip("mbi")
    from opendp.extras.mbi import _fixed

    original_search = _fixed.binary_search
    seen_bounds = []

    def search(predicate, *args, **kwargs):
        seen_bounds.append(kwargs.get("bounds"))
        assert predicate(0.0) is False
        return original_search(predicate, *args, **kwargs)

    monkeypatch.setattr(_fixed, "binary_search", search)

    domain = dp.lazyframe_domain(
        [dp.series_domain("A", dp.atom_domain(T="u32", bounds=(0, 1)))]
    )
    d_in = [dp.polars.Bound(per_group=1)]
    measurement = Fixed(queries=[Count(("A",))]).make_marginals(
        domain,
        dp.frame_distance(dp.symmetric_distance()),
        dp.zero_concentrated_divergence(),
        d_in=d_in,
        d_out=0.2,
        marginals=Marginals(),
        model=None,
    )

    assert seen_bounds == [(0.0, None)]
    assert measurement.check(d_in, 0.2)
