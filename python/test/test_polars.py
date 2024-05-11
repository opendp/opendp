import pytest
import opendp.prelude as dp


dp.enable_features("contrib", "honest-but-curious")


def seed(schema):
    pl = pytest.importorskip("polars")
    return pl.DataFrame(None, schema, orient="row").lazy()  # type: ignore[attr-defined]


def example_series():
    pl = pytest.importorskip("polars")
    return [
        dp.series_domain("A", dp.option_domain(dp.atom_domain(T=dp.f64))),
        dp.series_domain("B", dp.atom_domain(T=dp.i32)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=dp.String))),
        dp.series_domain("D", dp.atom_domain(T=dp.i32)),
    ], [
        pl.Series("A", [1.0] * 50, dtype=pl.Float64),
        pl.Series("B", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
        pl.Series("C", ["1"] * 49 + [None], dtype=pl.String),
        pl.Series("D", [2] * 50, dtype=pl.Int32),
    ]


def test_infer():
    pl = pytest.importorskip("polars")
    series = pl.Series("A", [1] * 100)
    assert dp.RuntimeType.infer(series) == dp.Series
    assert dp.RuntimeType.infer(pl.DataFrame(series)) == dp.DataFrame
    assert dp.RuntimeType.infer(pl.LazyFrame(series)) == dp.LazyFrame
    assert dp.RuntimeType.infer(pl.col("A")) == dp.Expr


def example_lf(margin=None, **kwargs):
    pl = pytest.importorskip("polars")
    domains, series = example_series()
    lf_domain, lf = dp.lazyframe_domain(domains), pl.LazyFrame(series)
    if margin is not None:
        lf_domain = dp.with_margin(lf_domain, by=margin, **kwargs)
    return lf_domain, lf


def test_expr_domain():
    lf_domain, _ = example_lf()
    dp.expr_domain(lf_domain)


def test_domains():
    example_lf(margin=["B"])
    example_lf(margin=["B"], public_info="keys", max_partition_length=50)
    example_lf(
        margin=["B"],
        public_info="lengths",
        max_partition_length=50,
        max_num_partitions=3,
        max_partition_contributions=2,
        max_influenced_partitions=1,
    )


# data loaders
@pytest.mark.parametrize("domain,series", zip(*example_series()))
def test_series_ffi(domain, series):
    """ensure that series can be passed to/from Rust"""
    pl_testing = pytest.importorskip("polars.testing")

    t_ident = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    pl_testing.assert_series_equal(t_ident(series), series)


def test_lazyframe_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    pl_testing = pytest.importorskip("polars.testing")
    lf_domain, lf = example_lf()
    t_ident = (lf_domain, dp.symmetric_distance()) >> dp.t.then_identity()

    pl_testing.assert_frame_equal(t_ident(lf).collect(), lf.collect())


def test_expr_ffi():
    """ensure that expr domain's carrier type can be passed to/from Rust"""
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    expr_domain = dp.expr_domain(lf_domain, grouping_columns=[])
    t_ident = (expr_domain, dp.symmetric_distance()) >> dp.t.then_identity()
    assert str(t_ident((lf, pl.col("A")))[1]) == str(pl.col("A"))


@pytest.mark.parametrize(
    "measure", [dp.max_divergence(T=float), dp.zero_concentrated_divergence(T=float)]
)
def test_private_lazyframe_explicit_sum(measure):
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf_domain, lf = example_lf(
        margin=["B"], public_info="keys", max_partition_length=50
    )

    expr = pl.col("A").fill_null(0.0).clip(0.0, 1.0).sum().dp.noise(0.0)
    print("expr", expr)
    plan = seed(lf.schema).group_by("B").agg(expr).sort("B")
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 0.0
    )

    df_exp = pl.DataFrame(
        [
            pl.Series("B", list(range(1, 6)), dtype=pl.Int32),
            pl.Series("A", [10.0] * 5),
        ]
    )
    df_act = m_lf(lf).collect()
    pl_testing.assert_frame_equal(df_act, df_exp)


@pytest.mark.parametrize(
    "measure", [dp.max_divergence(T=float), dp.zero_concentrated_divergence(T=float)]
)
def test_private_lazyframe_sum(measure):
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf_domain, lf = example_lf(
        margin=["B"], public_info="keys", max_partition_length=50
    )
    expr = pl.col("A").fill_null(0.0).dp.sum((1.0, 2.0), scale=0.0)
    plan = seed(lf.schema).group_by("B").agg(expr).sort("B")
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 0.0
    )

    expect = pl.DataFrame(
        [
            pl.Series("B", [1, 2, 3, 4, 5], dtype=pl.Int32),
            pl.Series("A", [10.0] * 5, dtype=pl.Float64),
        ]
    )
    pl_testing.assert_frame_equal(m_lf(lf).collect(), expect)


@pytest.mark.parametrize(
    "measure", [dp.max_divergence(T=float), dp.zero_concentrated_divergence(T=float)]
)
def test_private_lazyframe_mean(measure):
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf_domain, lf = example_lf(
        margin=["B"], public_info="lengths", max_partition_length=50
    )

    expr = pl.col("A").fill_null(0.0).dp.mean((1.0, 2.0), scale=0.0)
    plan = seed(lf.schema).group_by("B").agg(expr).sort("B")
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 1.0
    )

    expect = pl.DataFrame(
        [
            pl.Series("B", [1, 2, 3, 4, 5], dtype=pl.Int32),
            pl.Series("A", [1.0] * 5, dtype=pl.Float64),
        ]
    )
    pl_testing.assert_frame_equal(m_lf(lf).collect(), expect)


def test_stable_lazyframe():
    pl = pytest.importorskip("polars")
    lf_domain, lf = example_lf()
    with pytest.raises(dp.OpenDPException):
        dp.t.make_stable_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            lf.with_columns(pl.col("A").cast(int)),
        )


def test_stable_expr():
    pl = pytest.importorskip("polars")
    domain = dp.expr_domain(example_lf()[0])
    with pytest.raises(dp.OpenDPException):
        dp.t.make_stable_expr(domain, dp.symmetric_distance(), pl.col("A").cast(int))


def test_private_expr():
    pl = pytest.importorskip("polars")
    domain = dp.expr_domain(example_lf(margin=[])[0], grouping_columns=[])
    with pytest.raises(dp.OpenDPException):
        dp.m.make_private_expr(
            domain,
            dp.symmetric_distance(),
            dp.max_divergence(T=float),
            pl.col("A").sum(),
        )


def test_private_lazyframe_median():
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf_domain, lf = example_lf(
        margin=["A"], public_info="keys", max_partition_length=50
    )
    candidates = list(range(1, 6))
    expr = pl.col("B").dp.median(candidates, 1.0)
    plan = seed(lf.schema).group_by("A").agg(expr)
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan, 0.0
    )
    expect = pl.DataFrame(
        [pl.Series("A", [1.0], dtype=pl.Float64), pl.Series("B", [3], dtype=pl.Int64)]
    )

    pl_testing.assert_frame_equal(m_lf(lf).collect(), expect)


@pytest.mark.parametrize(
    "measure", [dp.max_divergence(T=float), dp.zero_concentrated_divergence(T=float)]
)
def test_filter(measure):
    """ensure that expr domain's carrier type can be passed to/from Rust"""
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf_domain, lf = example_lf(margin=[], public_info="keys", max_partition_length=50)

    plan = lf.filter(pl.col("B") < 2).select(pl.len().dp.noise(scale=0.0))

    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan
    )

    expect = pl.DataFrame([pl.Series("len", [10], dtype=pl.UInt32)])
    pl_testing.assert_frame_equal(m_lf(lf).collect(), expect)


def test_polars_context():
    pl = pytest.importorskip("polars")

    lf = pl.LazyFrame(
        {"A": [1, 2, 3, 4], "B": ["x", "x", "y", None]},
        schema={"A": pl.Int32, "B": pl.String},
    )

    context = dp.Context.compositor(
        data=lf,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=2,
        margins={
            # TODO: this is redundant with the second margin
            (): dp.Margin(max_partition_length=5),
            ("B",): dp.Margin(public_info="keys", max_partition_length=5),
        },
    )

    _df1 = (
        context.query()
        .with_columns(pl.col("B").is_null().alias("B_nulls"))
        .filter(pl.col("B_nulls"))
        .select(pl.col("A").fill_null(2.0).dp.sum((0, 3)))
        .release()
        .collect()
    )

    _df2 = (
        context.query()
        .group_by("B")
        .agg(
            pl.len().dp.noise(),
            pl.col("A").fill_null(2).dp.sum((0, 3))
        )
        .release()
        .collect()
    )
