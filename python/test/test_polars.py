import pytest
import opendp.prelude as dp
import os
import warnings


def test_polars_version():
    pl = pytest.importorskip("polars")
    from opendp.mod import _EXPECTED_POLARS_VERSION

    assert pl.__version__ == _EXPECTED_POLARS_VERSION


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
    plan = seed(lf.collect_schema()).group_by("B").agg(expr).sort("B")
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
    pl_testing.assert_frame_equal(df_act.sort("B"), df_exp)


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
    plan = seed(lf.collect_schema()).group_by("B").agg(expr).sort("B")
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 0.0
    )

    expect = pl.DataFrame(
        [
            pl.Series("B", [1, 2, 3, 4, 5], dtype=pl.Int32),
            pl.Series("A", [10.0] * 5, dtype=pl.Float64),
        ]
    )
    pl_testing.assert_frame_equal(m_lf(lf).collect().sort("B"), expect)


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
    plan = seed(lf.collect_schema()).group_by("B").agg(expr).sort("B")
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 1.0
    )

    expect = pl.DataFrame(
        [
            pl.Series("B", [1, 2, 3, 4, 5], dtype=pl.Int32),
            pl.Series("A", [1.0] * 5, dtype=pl.Float64),
        ]
    )
    pl_testing.assert_frame_equal(m_lf(lf).collect().sort("B"), expect)


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
    plan = seed(lf.collect_schema()).group_by("A").agg(expr)
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


def test_onceframe_multi_collect():
    pl = pytest.importorskip("polars")

    lf_domain, lf = example_lf()
    plan = seed(lf.collect_schema()).select(pl.len().dp.noise(0.0))
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan
    )

    of = m_lf(lf)
    of.collect()
    with pytest.raises(dp.OpenDPException):
        of.collect()


def test_onceframe_lazy():
    pl = pytest.importorskip("polars")

    lf_domain, lf = example_lf()
    plan = seed(lf.collect_schema()).select(pl.len().dp.noise(0.0))
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan
    )

    of = m_lf(lf)
    assert isinstance(of.lazy(), pl.LazyFrame)


@pytest.mark.parametrize(
    "measure", [dp.max_divergence(T=float), dp.zero_concentrated_divergence(T=float)]
)
def test_mechanisms(measure):
    pl_testing = pytest.importorskip("polars.testing")

    pl = pytest.importorskip("polars")

    lf_domain, lf = example_lf()

    if measure == dp.max_divergence(T=float):
        expr = pl.len().dp.laplace(0.0)
    else:
        expr = pl.len().dp.gaussian(0.0)

    plan = seed(lf.collect_schema()).select(expr)
    m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), measure, plan, 0.0
    )

    expect = pl.DataFrame([pl.Series("len", [50], dtype=pl.UInt32)])
    pl_testing.assert_frame_equal(m_lf(lf).collect(), expect)


def test_wrong_mechanism():
    pl = pytest.importorskip("polars")

    lf_domain, lf = example_lf()

    plan = seed(lf.collect_schema()).select(pl.len().dp.gaussian(0.0))
    with pytest.raises(dp.OpenDPException) as err:
        dp.m.make_private_lazyframe(
            lf_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan, 0.0
        )
    assert "expected Laplace distribution, found Gaussian" in (err.value.message or "")


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
            (): dp.polars.Margin(max_partition_length=5),
            ("B",): dp.polars.Margin(public_info="keys", max_partition_length=5),
        },
    )

    (
        context.query()
        .with_columns(pl.col("B").is_null().alias("B_nulls"))
        .filter(pl.col("B_nulls"))
        .select(pl.col("A").fill_null(2.0).dp.sum((0, 3)))
        .release()
        .collect()
    )

    (
        context.query()
        .group_by("B")
        .agg(pl.len().dp.noise(), pl.col("A").fill_null(2).dp.sum((0, 3)))
        .release()
        .collect()
    )


def test_polars_describe():
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})

    context = dp.Context.compositor(
        data=lf,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=2,
        margins={
            ("B",): dp.polars.Margin(public_info="keys", max_partition_length=5),
        },
    )

    expected = pl.DataFrame(
        {
            "column": ["len", "A", "B"],
            "aggregate": ["Len", "Sum", "Sum"],
            "distribution": ["Integer Laplace", "Integer Laplace", "Integer Laplace"],
            # * sensitivity of the count is 1 (adding/removing one row changes the count by at most one), 
            # * sensitivity of each sum is 3 (adding/removing one row with value as big as three...) 
            # Therefore the noise scale of the sum query should be 3x greater 
            # in order to consume the same amount of budget as the count.
            "scale": [6.0, 18.0, 18.0],
        }
    )

    summer = pl.col("A").fill_null(2).dp.sum((0, 3))

    query = (
        context.query()
        .group_by("B")
        .agg(pl.len().dp.noise(), summer, summer.alias("B"))
    )

    actual = query.summarize()
    pl_testing.assert_frame_equal(expected, actual)

    accuracy = [
        dp.discrete_laplacian_scale_to_accuracy(6.0, 0.05),
        dp.discrete_laplacian_scale_to_accuracy(18.0, 0.05),
        dp.discrete_laplacian_scale_to_accuracy(18.0, 0.05)
    ]
    expected = expected.hstack([pl.Series("accuracy", accuracy)])
    actual = query.summarize(alpha=0.05)
    pl_testing.assert_frame_equal(expected, actual)


def test_polars_accuracy_threshold():
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    context = dp.Context.compositor(
        data=pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=2,
        margins={
            ("B",): dp.polars.Margin(max_partition_length=5),
        },
    )

    expected = pl.DataFrame(
        {
            "column": ["len", "A"],
            "aggregate": ["Len", "Sum"],
            "distribution": ["Integer Laplace", "Integer Laplace"],
            "scale": [4.000000000000001, 12.000000000000004],
            "threshold": [65, None]
        },
        schema_overrides={"threshold": pl.UInt32}
    )

    query = (
        context.query()
        .group_by("B")
        .agg(pl.len().dp.noise(), pl.col("A").fill_null(2).dp.sum((0, 3)))
    )

    actual = query.summarize()
    pl_testing.assert_frame_equal(expected, actual)


def test_polars_non_wrapping():
    pl = pytest.importorskip("polars")

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": ["x", "x", "y", None]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    # only calls that return a LazyFrame or LazyGroupBy are wrapped
    assert context.query().explain() == 'DF ["A"]; PROJECT */1 COLUMNS; SELECTION: None'
    assert context.query().collect_schema() == {"A": pl.String}
    serial = context.query().with_columns(pl.col("A") + 2).serialize(format="json")
    assert serial.startswith('{"HStack":')
    # for coverage: attribute access works properly
    context.query()._ldf


def test_polars_collect_early():
    pl = pytest.importorskip("polars")

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": ["x", "x", "y", None]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )

    # catch the user trying to collect before release
    with pytest.raises(ValueError):
        context.query().collect()

    with pytest.raises(ValueError):
        context.query().describe()


def test_polars_threshold():
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    lf = pl.LazyFrame(
        {"A": [1] * 1000, "B": ["x"] * 500 + ["y"] * 500},
        schema={"A": pl.Int32, "B": pl.String},
    )

    context = dp.Context.compositor(
        data=lf,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
        split_evenly_over=2,
        margins={
            ("A",): dp.polars.Margin(public_info="keys"),
        },
    )

    actual = (
        context.query()
        .group_by("A")
        .agg(pl.len().dp.noise())
        .summarize()
    )

    expected = pl.DataFrame({
        "column": ["len"],
        "aggregate": ["Len"],
        "distribution": ["Integer Laplace"],
        "scale": [2.0]
    })

    # check that no threshold is set when keys are known
    pl_testing.assert_frame_equal(actual, expected)

    # check that query runs.
    print('output should be two columns ("A" and "len") with one row (1, ~1000)')
    print(
        context.query()
        .group_by("A")
        .agg(pl.len().dp.noise())
        .release()
        .collect()
    )

    actual = (
        context.query()
        .group_by("B")
        .agg(pl.len().dp.noise())
        .summarize()
    )

    expected = pl.DataFrame({
        "column": ["len"],
        "aggregate": ["Len"],
        "distribution": ["Integer Laplace"],
        "scale": [2.0],
        "threshold": [33]
    }, schema_overrides={"threshold": pl.UInt32})

    # threshold should work out to 33
    pl_testing.assert_frame_equal(actual, expected)

    # check that query runs.
    print('output should be two columns ("B" and "len") with two rows (1, ~500) each')
    print(
        context.query()
        .group_by("B")
        .agg(pl.len().dp.noise())
        .release()
        .collect()
    )

@pytest.mark.skipif(
    os.getenv('FORCE_TEST_REPLACE_BINARY_PATH') != "1", 
    reason="setting OPENDP_POLARS_LIB_PATH interferes with the execution of other tests"
)
def test_replace_binary_path():
    import os
    pl = pytest.importorskip("polars")
    expr = pl.len().dp.noise(scale=1.)

    # check that the library overwrites paths
    os.environ["OPENDP_POLARS_LIB_PATH"] = "testing!"

    m_expr = dp.m.make_private_expr(
        dp.expr_domain(example_lf()[0], grouping_columns=[]),
        dp.partition_distance(dp.symmetric_distance()),
        dp.max_divergence(T=float),
        expr,
    )

    assert str(m_expr((pl.LazyFrame(dict()), pl.all()))) == "len().testing!:noise_plugin()"

    # check that local paths in new expressions get overwritten
    os.environ["OPENDP_POLARS_LIB_PATH"] = __file__
    assert str(pl.len().dp.noise(scale=1.)) == f"len().{__file__}:noise([null, dyn float: 1.0])"

    # cleanup
    del os.environ["OPENDP_POLARS_LIB_PATH"]


def test_pickle_bomb():
    pl = pytest.importorskip("polars")

    from polars._utils.parse import parse_into_list_of_expressions  # type: ignore[import-not-found]
    from polars._utils.wrap import wrap_expr  # type: ignore[import-not-found]
    from opendp._lib import lib_path
    import io
    import re
    import pickle

    # modified from https://intoli.com/blog/dangerous-pickles/
    poison_binary = b'c__builtin__\neval\n(V1 / 0\ntR.'

    # poison_binary raises a ZeroDivisionError if it is ever unpickled
    with pytest.raises(ZeroDivisionError):
        pickle.loads(poison_binary)

    # craft an expression that contains a poisoned pickle binary

    py_exprs = parse_into_list_of_expressions((pl.len(), pl.lit("Laplace"), pl.lit(0.7)))

    # Replicates parts of register_plugin_function from Polars,
    # to allow injection of the specially-crafted pickle binary
    bomb_expr = wrap_expr(
        pl.polars.register_plugin_function(
            plugin_path=str(lib_path),
            function_name="noise",
            args=py_exprs,
            kwargs=poison_binary,
            is_elementwise=True,
            input_wildcard_expansion=False,
            returns_scalar=False,
            cast_to_supertype=False,
            pass_name_to_apply=False,
            changes_length=False,
        )
    )

    # craft a lazyframe that contains a poisoned pickle binary
    lf_domain, lf = example_lf()
    bomb_lf = lf.select(bomb_expr)

    # ensure that ser/de round-trip of expression does not trigger pickle
    ser_expr = bomb_expr.meta.serialize()
    pl.Expr.deserialize(io.BytesIO(ser_expr))

    # ensure that ser/de round-trip of lazyframe does not trigger pickle
    ser_lf = bomb_lf.serialize()
    pl.LazyFrame.deserialize(io.BytesIO(ser_lf))

    # OpenDP explicitly rejects any pickled data it finds
    err_msg_re = re.escape("OpenDP does not allow pickled keyword arguments as they may enable remote code execution.")
    with pytest.raises(dp.OpenDPException, match=err_msg_re):
        dp.m.make_private_expr(
            dp.expr_domain(lf_domain, grouping_columns=[]),
            dp.partition_distance(dp.symmetric_distance()),
            dp.max_divergence(T=float),
            bomb_expr,
        )

    with pytest.raises(dp.OpenDPException, match=err_msg_re):
        dp.m.make_private_lazyframe(
            lf_domain,
            dp.symmetric_distance(),
            dp.max_divergence(T=float),
            bomb_lf,
        )


def test_cut():
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")

    data = pl.LazyFrame({"x": [0.4, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5]})
    with warnings.catch_warnings():
        context = dp.Context.compositor(
            data=data.with_columns(pl.col("x").cut([1.0, 2.0, 3.0]).to_physical()),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=10000.0),
            split_evenly_over=1,
            margins={("x",): dp.polars.Margin(public_info="keys")},
        )
    actual = (
        context.query()
        .group_by("x")
        .agg(pl.len().dp.noise())
        .release()
        .collect()
        .sort("x")
    )
    expected = pl.DataFrame(
        {"x": [0, 1, 2, 3], "len": [2, 2, 2, 1]}, 
        schema={"x": pl.UInt32, "len": pl.UInt32},
    )

    pl_testing.assert_frame_equal(actual, expected)


@pytest.mark.xfail(raises=AssertionError)
def test_csv_bad_encoding_loading():
    # See https://github.com/opendp/opendp/issues/1976
    # If a CSV is misencoded, Polars loads an empty dataframe.
    # Since we tell users not to look at their data,
    # we may want to try harder to load the csv,
    # or give more information on failure.
    pl = pytest.importorskip("polars")
    pl_testing = pytest.importorskip("polars.testing")
    import tempfile

    name = 'André'
    name_b = name.encode('iso-8859-1') # Polars only handles 'utf-8'.

    with tempfile.NamedTemporaryFile(delete=False) as fp:
        # By default, would delete file on "close()";
        # With "delete=False", clean up when exiting "with" instead.
        fp.write(b'name\n' + name_b)
        fp.close()
        df = pl.scan_csv(fp.name, ignore_errors=True)
        expected = pl.LazyFrame(
            {"name": [name]},
            schema={"name": pl.String},
        )
        pl_testing.assert_frame_equal(df, expected)