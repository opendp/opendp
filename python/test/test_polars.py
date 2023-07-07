import pytest
import opendp.prelude as dp
import polars as pl

dp.enable_features("contrib", "honest-but-curious")

# domains
def test_series_domain():
    return [
        dp.series_domain("A", dp.atom_domain(T=float)),
        dp.series_domain("B", dp.atom_domain(T=int)),
        dp.series_domain("C", dp.option_domain(dp.atom_domain(T=str))),
    ], {
        "A": [1.0] * 50,
        "B": [1] * 50,
        "C": ["1"] * 50,
    }

def test_lazyframe_domain():
    domains, data = test_series_domain()
    return dp.lazyframe_domain(domains), pl.LazyFrame(data, schema_overrides={"B": pl.Int32})

def test_dataframe_domain():
    domains, data = test_series_domain()
    return dp.dataframe_domain(domains), pl.DataFrame(data={"B": pl.Int32})

def test_lazyframe_domain_with_counts():
    counts = pl.LazyFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32, "counts": pl.UInt32})
    domain, data = test_lazyframe_domain()
    return domain.with_counts(counts), data

def test_dataframe_domain_with_counts():
    counts = pl.DataFrame({"B": [1], "counts": [50]}, schema_overrides={"B": pl.Int32, "counts": pl.UInt32})
    domain, data = test_dataframe_domain()
    return domain.with_counts(counts), data

def test_expr_domain():
    lf_domain = test_lazyframe_domain()[0]
    expr_domain = dp.expr_domain(lf_domain, grouping_columns=["A"], active_column="B")

    dp.expr_domain(lf_domain, context="filter", active_column="B")
    with pytest.raises(dp.OpenDPException):
        dp.expr_domain(lf_domain, context="X", active_column="B")

    # TODO: data loader for ExprDomain members
    return expr_domain, None


# data loaders
def test_series_ffi():
    """ensure that series can be passed to/from Rust"""
    domains, datas = test_series_domain()
    for domain, (name, data) in zip(domains, datas.items()):
        ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
        print(ident_trans(pl.Series(name, data)))

def test_lazyframe_ffi():
    """ensure that lazyframes can be passed to/from Rust"""
    domain, data = test_lazyframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))

def test_dataframe_ffi():
    """ensure that dataframes can be passed to/from Rust"""
    domain, data = test_dataframe_domain()
    ident_trans = (domain, dp.symmetric_distance()) >> dp.t.then_identity()
    print(ident_trans(data))

# constructors
def test_scan_csv():
    df_domain = dp.lazyframe_domain([dp.series_domain("A", dp.atom_domain(T=float))])
    input_space = dp.csv_domain(df_domain), dp.symmetric_distance()

    scanner = input_space >> dp.t.then_scan_csv()
    with pytest.raises(dp.OpenDPException) as err:
        scanner("A/B.csv")
    

def test_collect_lazy():
    domain, data = test_lazyframe_domain()
    space = domain, dp.symmetric_distance()
    trans_lazy = space >> dp.t.then_collect() >> dp.t.then_lazy()
    trans_lazy(data)


def test_make_with_columns():
    domain, data = test_lazyframe_domain()
    metric = dp.symmetric_distance()
    expr_domain = dp.expr_domain(domain, context="with_columns")

    trans_lazy = (domain, metric) >> dp.t.then_with_columns([
        (expr_domain, metric) >> dp.t.then_col("A") >> dp.t.then_clamp_expr((1., 2.))
    ])
    # trans_lazy(data)

def test_private_mean_expr():
    domain, data = test_lazyframe_domain()
    metric = dp.symmetric_distance()
    expr_domain = dp.expr_domain(domain, context="with_columns")
    expr_metric = dp.l1(dp.symmetric_distance())

    # Fail because no margin
    with pytest.raises(dp.OpenDPException):
        meas_lazy = (domain, metric) >> dp.t.then_with_columns([
            (expr_domain, metric) >> dp.t.then_col("A") >> dp.m.then_private_mean_expr(0.5)
        ])

    # now add margins, it should pass
    a_counts = pl.LazyFrame(
        {"A": [1.0], "counts": [50]},
        schema_overrides={"A": pl.Float64, "counts": pl.UInt32},
    )
    b_counts = pl.LazyFrame(
        {"B": [1], "counts": [50]},
        schema_overrides={"B": pl.Int32, "counts": pl.UInt32},
    )

    domain = domain.with_counts(a_counts).with_counts(b_counts)
    expr_domain = dp.expr_domain(domain, grouping_columns=["A"])
    meas_lazy = (
        (domain, metric)
        >> dp.t.then_groupby_stable(["A"])
        >> dp.m.then_private_agg(
            dp.c.make_basic_composition(
                [
                    (expr_domain, expr_metric)
                    >> dp.t.then_col("B")
                    >> dp.t.then_clamp_expr((2, 3))
                    >> dp.m.then_private_mean_expr(0.5)
                ]
            )
        )
        >> dp.t.make_collect(domain, metric)
    )

    print(meas_lazy(data))


test_private_mean_expr()
