"""Polars DataFrame domains and Python-backed stable frame conversions."""

import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


def _frame():
    return pl.DataFrame(
        {
            "age": pl.Series([20, 30, 40], dtype=pl.Int32),
            "region": pl.Series(["east", "west", "east"], dtype=pl.String),
        }
    )


def _domain():
    return dp.dataframe_domain(
        [
            dp.series_domain("age", dp.atom_domain(T=dp.i32)),
            dp.series_domain("region", dp.atom_domain(T=str)),
        ]
    )


def test_dataframe_domain_ffi_and_inference():
    frame = _frame()
    domain = _domain()
    assert isinstance(domain, dp.DataFrameDomain)
    assert domain.columns == ["age", "region"]
    assert domain.get_series_domain("age").element_domain.carrier_type == dp.i32
    # FrameDomain.member is intentionally unavailable in polars-ffi builds.
    assert isinstance(dp.domain_of(frame, infer=True), dp.DataFrameDomain)
    inferred = dp.domain_of(frame, infer=True)
    assert inferred.columns == ["age", "region"]
    # A schema does not establish a non-null constraint.
    assert all(inferred.get_series_domain(name).nullable for name in inferred.columns)

    assert isinstance(dp.domain_of(frame.lazy(), infer=True), dp.LazyFrameDomain)
    lazy_domain = dp.domain_of(frame.lazy(), infer=True)
    assert lazy_domain.columns == domain.columns
    region = lazy_domain.get_series_domain("region")
    assert region.name == "region"
    assert region.element_domain.carrier_type == "String"
    # LazyFrame schema inference cannot certify the absence of nulls.
    assert region.nullable


def test_collect_and_lazy_are_stable_python_transformations():
    from opendp.extras.polars import make_collect, make_lazy

    frame = _frame()
    domain = _domain()
    metric = dp.symmetric_distance()
    lazy = make_lazy(domain, metric)
    assert isinstance(lazy.output_domain, dp.LazyFrameDomain)
    assert lazy.map(2) == 2
    lazy_frame = lazy(frame)
    assert isinstance(lazy_frame, pl.LazyFrame)
    assert lazy_frame.collect().equals(frame)
    collect = make_collect(lazy.output_domain, metric)
    assert isinstance(collect.output_domain, dp.DataFrameDomain)
    assert collect.map(2) == 2
    result = collect(lazy_frame)
    assert result.schema == frame.schema
    assert result.equals(frame)
    assert (lazy >> collect).map(2) == 2


def test_with_margin_supports_eager_and_lazy_frame_domains():
    from opendp.extras.polars import make_collect, make_lazy

    domain = _domain()
    margin = dp.polars.Margin(by=["region"], max_groups=2)
    eager = dp.with_margin(domain, margin)
    assert isinstance(eager, dp.DataFrameDomain)
    assert eager.columns == domain.columns

    metric = dp.symmetric_distance()
    lazy = make_lazy(eager, metric)
    assert lazy(_frame()).collect().equals(_frame())
    assert make_collect(lazy.output_domain, metric).output_domain == eager

    separate_lazy = dp.with_margin(
        dp.lazyframe_domain(
            [domain.get_series_domain(name) for name in domain.columns]
        ),
        margin,
    )
    assert isinstance(separate_lazy, dp.LazyFrameDomain)
    assert separate_lazy == lazy.output_domain


def test_frame_distance_stability():
    from opendp.extras.polars import make_collect, make_lazy

    metric = dp.frame_distance(dp.symmetric_distance())
    domain = _domain()
    lazy = make_lazy(domain, metric)
    assert lazy.output_metric == metric
    assert lazy.map([dp.polars.Bound(by=[], per_group=2, num_groups=1)]) == [
        dp.polars.Bound(by=[], per_group=2, num_groups=1)
    ]
    collect = make_collect(lazy.output_domain, metric)
    assert collect.output_metric == metric


def test_query_collect_and_lazy_do_not_execute_or_release_rows():
    frame = _frame()
    ctx = dp.Context.compositor(
        frame,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_domain(),
        split_evenly_over=1,
    )
    lazy = ctx.query().lazy()
    assert isinstance(lazy, dp.polars.LazyFrameQuery)
    assert ctx.current_privacy_loss() == []
    # Only the Polars query wrapper exposes the special collect operation.
    assert "collect" not in dp.Query.__dict__
    collected = lazy.collect()
    assert isinstance(collected, dp.Query)
    chain = collected._chain
    assert isinstance(chain, dp.Transformation)
    assert isinstance(chain.output_domain, dp.DataFrameDomain)
    assert chain.map(1) == [dp.polars.Bound(by=[], per_group=1, num_groups=None)]
    assert ctx.current_privacy_loss() == []


def test_lazyframe_query_collect_compiles_selection():
    frame = _frame()
    ctx = dp.Context.compositor(
        frame.lazy(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    collected = ctx.query().select("age").collect()
    chain = collected._chain
    assert isinstance(chain, dp.Transformation)
    assert isinstance(chain.output_domain, dp.DataFrameDomain)
    assert chain.output_domain.columns == ["age"]
    assert ctx.current_privacy_loss() == []


@pytest.mark.parametrize(
    "unit,expected",
    [
        (
            lambda: dp.unit_of(contributions=1),
            lambda: dp.frame_distance(dp.symmetric_distance()),
        ),
        (
            lambda: dp.unit_of(changes=1),
            lambda: dp.frame_distance(dp.symmetric_distance()),
        ),
    ],
)
def test_query_collect_respects_input_metric(unit, expected):
    from opendp.domains import lazyframe_domain

    frame = _frame().lazy()
    domain = lazyframe_domain(
        [_domain().get_series_domain(name) for name in _domain().columns]
    )
    ctx = dp.Context.compositor(
        frame,
        unit(),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    collected = ctx.query().select("age").collect()
    transformation = collected.resolve(allow_transformations=True)
    assert transformation.output_metric == expected()
    assert isinstance(transformation.output_domain, dp.DataFrameDomain)
    assert ctx.current_privacy_loss() == []


@pytest.mark.parametrize(
    "unit",
    [
        lambda: dp.unit_of(contributions=1, ordered=True),
        lambda: dp.unit_of(changes=1, ordered=True),
    ],
)
def test_query_collect_rejects_ordered_metric(unit):
    # Ordered Polars adjacency is rejected before a query can be constructed.
    with pytest.raises(
        ValueError, match="Ordered metrics are not supported for Polars"
    ):
        dp.Context.compositor(
            _frame().lazy(),
            unit(),
            dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )


def test_eager_fill_null_establishes_non_nullable_domain():
    from opendp.extras.polars import make_dataframe_fill_null

    frame = pl.DataFrame(
        {
            "left": pl.Series([None, 2.0], dtype=pl.Float64),
            "right": pl.Series([1.0, None], dtype=pl.Float64),
        }
    )
    domain = dp.domain_of(frame, infer=True)
    assert all(domain.get_series_domain(n).nullable for n in domain.columns)
    fill = make_dataframe_fill_null(domain, dp.symmetric_distance(), value=0)
    result = fill(frame)
    assert result.to_dict(as_series=False) == {
        "left": [0.0, 2.0],
        "right": [1.0, 0.0],
    }
    assert result.schema == frame.schema
    assert all(
        not fill.output_domain.get_series_domain(n).nullable for n in domain.columns
    )
    assert fill.map(2) == 2

    ctx = dp.Context.compositor(
        frame,
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=domain,
        split_evenly_over=1,
    )
    query = ctx.query().fill_null(value=0)
    assert query.resolve(allow_transformations=True).map(1) == 1
    assert ctx.current_privacy_loss() == []


def test_eager_fill_null_preserves_other_columns_and_rejects_bad_values():
    from opendp.extras.polars import make_dataframe_fill_null

    frame = pl.DataFrame(
        {
            "numeric": pl.Series([None, 2], dtype=pl.Int32),
            "text": pl.Series([None, "ok"], dtype=pl.String),
        }
    )
    domain = dp.domain_of(frame, infer=True)
    fill = make_dataframe_fill_null(
        domain,
        dp.symmetric_distance(),
        value=0,
        columns="numeric",
    )
    output = fill(frame)
    assert output["numeric"].to_list() == [0, 2]
    assert output["text"].to_list() == [None, "ok"]
    assert not fill.output_domain.get_series_domain("numeric").nullable
    assert fill.output_domain.get_series_domain("text").nullable
    assert output.schema == frame.schema

    with pytest.raises(ValueError, match="Invalid fill value"):
        make_dataframe_fill_null(
            domain,
            dp.symmetric_distance(),
            value="wrong",
            columns="numeric",
        )
    with pytest.raises(ValueError, match="columns"):
        make_dataframe_fill_null(
            domain,
            dp.symmetric_distance(),
            value=0,
            columns="missing",
        )


def test_polars_dtype_utility_accepts_opendp_and_numpy_names():
    from opendp.extras.polars._dtypes import _polars_dtype

    assert _polars_dtype("i32") == _polars_dtype("int32") == pl.Int32
    assert _polars_dtype("f64") == _polars_dtype("float64") == pl.Float64
    assert _polars_dtype("String") == pl.String
    with pytest.raises(TypeError, match="Unsupported Polars dtype"):
        _polars_dtype("unsupported")


def test_lazy_query_boundary_conditions():
    context = dp.Context.compositor(
        _frame(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_domain(),
        split_evenly_over=1,
    )
    query = context.query()
    with pytest.raises(ValueError, match="stable"):
        query.new_with(chain=None).lazy()
    with pytest.raises(TypeError, match="DataFrame"):
        query.new_with(chain=(dp.atom_domain(T=dp.i32), dp.symmetric_distance())).lazy()

    lazy_domain = dp.lazyframe_domain(
        [_domain().get_series_domain(name) for name in _domain().columns]
    )
    assert isinstance(
        query.new_with(chain=(lazy_domain, dp.symmetric_distance())).lazy(),
        dp.polars.LazyFrameQuery,
    )

    from opendp.extras.polars import LazyFrameQuery

    lazy = query.lazy()
    plan = object.__getattribute__(lazy, "polars_plan")
    invalid = LazyFrameQuery(plan, query.new_with(chain=None))
    with pytest.raises(ValueError, match=r"stable (query|transformation) prefix"):
        invalid.collect()
    with pytest.raises(ValueError, match=r"stable (query|transformation) prefix"):
        invalid.resolve()
    with pytest.raises(ValueError, match="Unsupported metric for collect"):
        LazyFrameQuery(
            plan,
            query.new_with(chain=(lazy_domain, dp.discrete_distance())),
        ).collect()

    # A stable prefix can also enter the Polars aggregator after an eager
    # collect and a second lazy conversion without releasing private rows.
    collected = lazy.collect()
    resumed = collected.lazy()
    assert resumed.select(dp.len(signed=True)).resolve().input_domain == _domain()
    assert context.current_privacy_loss() == []


def test_fill_null_error_and_identity_cases():
    from opendp.extras.polars import make_dataframe_fill_null

    metric = dp.symmetric_distance()
    with pytest.raises(TypeError, match="DataFrameDomain"):
        make_dataframe_fill_null(dp.atom_domain(T=dp.i32), metric, 0)
    with pytest.raises(ValueError, match="row-adjacency"):
        make_dataframe_fill_null(_domain(), dp.discrete_distance(), 0)

    frame = _frame()
    identity = make_dataframe_fill_null(_domain(), metric, 0)
    assert identity(frame).equals(frame)
    assert identity.map(2) == 2

    nullable = dp.dataframe_domain(
        [
            dp.series_domain(
                "bounded", dp.option_domain(dp.atom_domain(T=dp.i32, bounds=(0, 1)))
            )
        ]
    )
    with pytest.raises(ValueError, match="outside the domain"):
        make_dataframe_fill_null(nullable, metric, 2, columns=("bounded",))
    with pytest.raises(ValueError, match="distinct"):
        make_dataframe_fill_null(nullable, metric, 0, columns=("bounded", "bounded"))


def test_fill_null_rejects_nonatomic_columns():
    from opendp.extras.polars import make_dataframe_fill_null

    domain = dp.dataframe_domain(
        [dp.series_domain("category", dp.option_domain(dp.categorical_domain()))]
    )
    with pytest.raises(TypeError, match="atomic columns"):
        make_dataframe_fill_null(domain, dp.symmetric_distance(), "other")


def test_contingency_table_stable_prefix_forms(monkeypatch):
    from opendp._internal import _make_measurement, _make_transformation
    import opendp.extras.polars as extra
    from opendp.extras.polars import LazyFrameQuery
    from opendp.extras.polars.contingency_table import ContingencyTableQuery

    def stable(input_domain, input_metric, *, lazyframe, MO):
        return _make_transformation(
            input_domain,
            input_metric,
            input_domain,
            input_metric,
            lambda data: data,
            lambda d: d,
        )

    def table(*, input_domain, input_metric, output_measure, **kwargs):
        return (
            _make_measurement(
                input_domain,
                input_metric,
                output_measure,
                lambda data: None,
                lambda d: 0,
            ),
            None,
            None,
        )

    monkeypatch.setattr(extra, "make_stable_lazyframe", stable)
    monkeypatch.setattr(extra, "make_contingency_table", table)
    context = dp.Context.compositor(
        _frame(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_domain(),
        split_evenly_over=1,
    )
    lazy = context.query().lazy()
    query = object.__getattribute__(lazy, "_query")
    plan = object.__getattribute__(lazy, "polars_plan")
    assert isinstance(lazy.contingency_table(), ContingencyTableQuery)

    domain, metric = query._chain.output_space
    direct = LazyFrameQuery(plan, query.new_with(chain=(domain, metric)))
    assert isinstance(direct.contingency_table(), ContingencyTableQuery)

    invalid = LazyFrameQuery(plan, query.new_with(chain=None))
    with pytest.raises(ValueError, match="stable query prefix"):
        invalid.contingency_table()


def test_collect_frame_distance_metric():
    from opendp.extras.polars import LazyFrameQuery

    context = dp.Context.compositor(
        _frame(),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        domain=_domain(),
        split_evenly_over=1,
    )
    lazy = context.query().lazy()
    plan = object.__getattribute__(lazy, "polars_plan")
    query = object.__getattribute__(lazy, "_query")
    domain = query._chain.output_domain
    metric = dp.frame_distance(dp.symmetric_distance())
    collected = LazyFrameQuery(plan, query.new_with(chain=(domain, metric))).collect()
    from opendp.mod import Transformation

    assert isinstance(collected.resolve(allow_transformations=True), Transformation)


def test_frame_conversion_rejects_trimmed_metrics():
    from opendp.extras.polars import make_collect, make_lazy

    domain = _domain()
    lazy_domain = dp.lazyframe_domain(
        [domain.get_series_domain(name) for name in domain.columns]
    )
    for metric in (
        dp.insert_delete_distance(),
        dp.hamming_distance(),
        dp.discrete_distance(),
    ):
        with pytest.raises(ValueError, match="Unsupported Polars frame metric"):
            make_lazy(domain, metric)
        with pytest.raises(ValueError, match="Unsupported Polars frame metric"):
            make_collect(lazy_domain, metric)


def test_frame_conversion_rejects_wrong_domain():
    from opendp.extras.polars import make_collect, make_lazy

    metric = dp.symmetric_distance()
    with pytest.raises(TypeError, match="Expected DataFrameDomain"):
        make_lazy(dp.lazyframe_domain([]), metric)
    with pytest.raises(TypeError, match="Expected LazyFrameDomain"):
        make_collect(dp.dataframe_domain([]), metric)


def test_fill_null_preserves_unaffected_margins():
    from opendp.extras.polars import make_dataframe_fill_null, make_lazy

    domain = dp.dataframe_domain(
        [
            dp.series_domain("region", dp.atom_domain(T=str)),
            dp.series_domain("number", dp.option_domain(dp.atom_domain(T=dp.i32))),
        ]
    )
    domain = dp.with_margin(domain, dp.polars.Margin(by=["region"], max_groups=2))
    domain = dp.with_margin(domain, dp.polars.Margin(by=["number"], max_groups=3))

    filled = make_dataframe_fill_null(
        domain, dp.symmetric_distance(), 0, columns="number"
    )
    output = make_lazy(filled.output_domain, dp.symmetric_distance()).output_domain
    assert output.get_margin(["region"]).max_groups == 2
    assert output.get_margin(["number"]).max_groups is None

    converted = make_lazy(domain, dp.symmetric_distance())
    assert converted.output_domain.get_margin(["region"]).max_groups == 2
    assert converted.output_domain.get_margin(["number"]).max_groups == 3


def test_fill_null_rejects_identifier_metrics():
    from opendp.extras.polars import make_dataframe_fill_null

    metric = dp.symmetric_id_distance(pl.col("region"))
    with pytest.raises(ValueError, match="row-adjacency"):
        make_dataframe_fill_null(_domain(), metric, 0)


def test_lazy_rejects_partially_specified_query():
    query = dp.Query(
        chain=dp.PartialChain(lambda _: None),
        output_measure=dp.max_divergence(),
    )
    with pytest.raises(ValueError, match="all arguments"):
        query.lazy()


def test_domain_of_polars_series():
    series = pl.Series("value", [1, 2, 3], dtype=pl.Int32)
    domain = dp.domain_of(series, infer=True)
    assert domain.name == "value"
    assert domain.element_domain.carrier_type == dp.i32
    assert domain.nullable
