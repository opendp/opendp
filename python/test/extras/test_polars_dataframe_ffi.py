"""Rust-backed Polars DataFrame FFI and stable frame conversions."""

import pytest

import opendp.prelude as dp

pl = pytest.importorskip("polars")
dp.enable_features("contrib")


def _frame():
    return pl.DataFrame({
        "age": pl.Series([20, 30, 40], dtype=pl.Int32),
        "region": pl.Series(["east", "west", "east"], dtype=pl.String),
    })


def _domain():
    return dp.dataframe_domain([
        dp.series_domain("age", dp.atom_domain(T=dp.i32)),
        dp.series_domain("region", dp.atom_domain(T=str)),
    ])


def test_dataframe_domain_ffi_and_inference():
    frame = _frame()
    domain = _domain()
    assert isinstance(domain, dp.DataFrameDomain)
    assert domain.columns == ["age", "region"]
    assert domain.get_series_domain("age").element_domain.carrier_type == dp.i32
    assert domain.member(frame)
    assert not domain.member(frame.lazy())
    assert isinstance(dp.domain_of(frame, infer=True), dp.DataFrameDomain)
    inferred = dp.domain_of(frame, infer=True)
    assert inferred.columns == ["age", "region"]
    # A schema does not establish a non-null constraint.
    assert all(inferred.get_series_domain(name).nullable for name in inferred.columns)
    assert inferred.member(frame)

    assert isinstance(dp.domain_of(frame.lazy(), infer=True), dp.LazyFrameDomain)
    lazy_domain = dp.domain_of(frame.lazy(), infer=True)
    assert lazy_domain.columns == domain.columns
    assert lazy_domain.get_series_domain("region") == domain.get_series_domain("region")


def test_collect_and_lazy_are_rust_transformations():
    from opendp.transformations import make_collect, make_lazy

    frame = _frame()
    domain = _domain()
    metric = dp.symmetric_distance()
    lazy = make_lazy(domain, metric)
    assert isinstance(lazy.output_domain, dp.LazyFrameDomain)
    assert lazy.map(2) == 2
    lazy_frame = lazy(frame)
    assert lazy.output_domain.member(lazy_frame)
    collect = make_collect(lazy.output_domain, metric)
    assert isinstance(collect.output_domain, dp.DataFrameDomain)
    assert collect.map(2) == 2
    result = collect(lazy_frame)
    assert collect.output_domain.member(result)
    assert result.equals(frame)
    assert (lazy >> collect).map(2) == 2


def test_with_margin_supports_eager_and_lazy_frame_domains():
    from opendp.transformations import make_collect, make_lazy

    domain = _domain()
    margin = dp.polars.Margin(by=["region"], max_groups=2)
    eager = dp.with_margin(domain, margin)
    assert isinstance(eager, dp.DataFrameDomain)
    assert eager.member(_frame())

    metric = dp.symmetric_distance()
    lazy = make_lazy(eager, metric)
    assert lazy.output_domain.member(lazy(_frame()))
    assert make_collect(lazy.output_domain, metric).output_domain == eager

    separate_lazy = dp.with_margin(
        dp.lazyframe_domain([
            domain.get_series_domain(name) for name in domain.columns
        ]),
        margin,
    )
    assert isinstance(separate_lazy, dp.LazyFrameDomain)
    assert separate_lazy == lazy.output_domain


def test_frame_distance_stability():
    from opendp.transformations import make_collect, make_lazy

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
        frame, dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0), domain=_domain(),
        split_evenly_over=1,
    )
    lazy = ctx.query().lazy()
    assert isinstance(lazy, dp.polars.LazyFrameQuery)
    assert ctx.current_privacy_loss() == []
    # Only the Polars query wrapper exposes the special collect operation.
    assert "collect" not in dp.Query.__dict__
    collected = lazy.collect()
    assert isinstance(collected, dp.Query)
    assert isinstance(collected._chain.output_domain, dp.DataFrameDomain)
    assert collected._chain.map(1) == 1
    assert ctx.current_privacy_loss() == []


def test_lazyframe_query_collect_compiles_selection():
    frame = _frame()
    ctx = dp.Context.compositor(
        frame.lazy(), dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0), split_evenly_over=1,
    )
    collected = ctx.query().select("age").collect()
    assert isinstance(collected._chain.output_domain, dp.DataFrameDomain)
    assert collected._chain.output_domain.columns == ["age"]
    assert ctx.current_privacy_loss() == []


@pytest.mark.parametrize("unit,expected", [
    (lambda: dp.unit_of(contributions=1),
     lambda: dp.frame_distance(dp.symmetric_distance())),
    (lambda: dp.unit_of(contributions=1, ordered=True),
     lambda: dp.frame_distance(dp.insert_delete_distance())),
    (lambda: dp.unit_of(changes=1),
     lambda: dp.frame_distance(dp.symmetric_distance())),
    (lambda: dp.unit_of(changes=1, ordered=True),
     lambda: dp.frame_distance(dp.insert_delete_distance())),
])
def test_query_collect_respects_input_metric(unit, expected):
    from opendp.domains import lazyframe_domain

    frame = _frame().lazy()
    domain = lazyframe_domain([
        _domain().get_series_domain(name) for name in _domain().columns
    ])
    ctx = dp.Context.compositor(
        frame, unit(), dp.loss_of(epsilon=1.0),
        domain=domain, split_evenly_over=1,
    )
    collected = ctx.query().select("age").collect()
    transformation = collected.resolve(allow_transformations=True)
    assert transformation.output_metric == expected()
    assert isinstance(transformation.output_domain, dp.DataFrameDomain)
    assert ctx.current_privacy_loss() == []
