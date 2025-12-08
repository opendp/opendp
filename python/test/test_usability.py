import re
import pytest
import opendp.prelude as dp


@pytest.mark.xfail(raises=dp.UnknownTypeException)
def test_iterable_data():
    # Currently fails with:
    #   opendp.mod.UnknownTypeException: <class 'range'>
    # Possible resolution:
    #   The data kwarg accepts iterables.
    context = dp.Context.compositor(
        data=range(100),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    sum_query = context.query().clamp((1.0, 10.0)).sum()
    sum_query.laplace()


def test_int_data_laplace_param():
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    sum_query = context.query().clamp((1, 10)).sum()
    sum_query.laplace(100)


@pytest.mark.xfail(raises=dp.OpenDPException)
def test_mean_without_size():
    # Currently fails with:
    #   opendp.mod.OpenDPException:
    #     MakeTransformation("dataset size must be known. Either specify size in the input domain or use make_resize")
    # Possible resolution:
    #   Error message suggests fixes in terms on the new Context API.
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0, 4.0, 5.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    mean_query = context.query().clamp((1.0, 10.0)).mean()
    mean_query.laplace()


@pytest.mark.xfail(raises=dp.OpenDPException)
def test_int_mean():
    # Currently fails with:
    #   opendp.mod.OpenDPException:
    #     FFI("No match for concrete type i32. You've got a debug binary! Debug binaries support fewer types. Consult https://docs.opendp.org/en/stable/contributing/development-environment.html#build-opendp")
    # Possible resolution:
    #   Should just be the same as any mean without a size.
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    mean_query = context.query().clamp((1, 10)).mean()
    mean_query.laplace()


def test_scalar_instead_of_vector():
    with pytest.raises(TypeError, match='To fix, wrap domain kwarg with dp.vector_domain()'):
        dp.Context.compositor(
            data=[1, 2, 3, 4, 5],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            domain=dp.domain_of(int),
        )

def test_query_dir():
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    query_dir = dir(context.query())
    assert 'count' in query_dir
    assert 'laplace' in query_dir


def test_string_instead_of_tuple_for_margin_key():
    pl = pytest.importorskip("polars")

    lf = pl.LazyFrame(
        {"a_column": [1, 2, 3, 4]},
        schema={"a_column": pl.Int32},
    )

    with pytest.raises(ValueError, match=re.escape('by (a_column) must be a sequence type; Did you mean ["a_column"]?')):
        dp.Context.compositor(
            data=lf,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            margins=[
                # To reproduce failure, the column name must be multiple characters.
                # TODO: We want to fail earlier because the key is not a tuple.
                # (mypy does catch this, so we need "type: ignore", but we can't rely on users running mypy.)
                dp.polars.Margin(by=("a_column"), invariant="keys", max_length=5), # type: ignore
            ],
        )


def test_margins_dict_instead_of_list():
    pl = pytest.importorskip("polars")

    lf = pl.LazyFrame(
        {"col": [1, 2, 3, 4]},
        schema={"col": pl.Int32},
    )

    with pytest.warns(DeprecationWarning, match=re.escape('Margin dicts should be replaced with lists, with the key supplied as the "by" kwarg')):
        dp.Context.compositor(
            data=lf,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            margins={ # type: ignore[arg-type]
                ('col',): dp.polars.Margin(invariant="keys", max_length=5),
            }
        )


@pytest.mark.parametrize(
    "domain", [dp.lazyframe_domain([]), dp.series_domain("A", dp.atom_domain(T=bool))])
def test_polars_data_loader_error_is_human_readable(domain):
    pytest.importorskip("polars")
    overall_pipeline = dp.c.make_adaptive_composition(
        domain, dp.symmetric_distance(), dp.max_divergence(), d_in=1,
        d_mids=[1.])
    with pytest.raises(ValueError, match="expected Polars *"):
        overall_pipeline("I'm not the right type!")


def test_polars_expr_loader_error_is_human_readable():
    pl = pytest.importorskip("polars")
    with pytest.raises(ValueError, match="expected Polars Expr"):
        dp.m.make_private_expr(
            dp.wild_expr_domain([]),
            dp.symmetric_distance(),
            dp.max_divergence(),
            pl.LazyFrame({}),
        )


def test_unrecognized_column():
    pl = pytest.importorskip("polars")

    context = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1, 2, 3, 4], "B": [5, 6, 7, 8]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
        split_evenly_over=1,
        margins=[]
    )
    config = pl.col("X").dp.mean((0, 10))

    plain_query = context.query().select(config)
    with pytest.raises(dp.OpenDPException, match=r"unrecognized column 'X' in output domain; expected one of: A, B"):
        plain_query.release()

def test_without_max_partition_length():
    pl = pytest.importorskip("polars")

    context_wo_margin = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1, 2, 3, 4], "B": [5, 6, 7, 8]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
        split_evenly_over=2,
        margins=[],
    )

    config = pl.col("A").dp.mean((0, 10))

    plain_query = context_wo_margin.query().select(config)
    with pytest.raises(dp.OpenDPException, match=r"must specify 'max_length' in a margin with by=\[\]"):
        plain_query.release()

    agg_query = context_wo_margin.query().group_by(["B"]).agg(config)
    with pytest.raises(dp.OpenDPException, match="must specify 'max_length' in a margin with by=\\[col\\(\"B\"\\)\\]"):
        agg_query.release()
    
# Add a margin and try the same queries again:
def test_with_max_length():
    pl = pytest.importorskip("polars")

    context_w_margin = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1, 2, 3, 4], "B": [5, 6, 7, 8]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
        split_evenly_over=2,
        margins=[dp.polars.Margin(
            by=[],
            max_length=1
        )],
    )

    config = pl.col("A").dp.mean((0, 10))

    plain_query = context_w_margin.query().select(config)
    plain_query.release()

    agg_query = context_w_margin.query().group_by(["B"]).agg(config)
    with pytest.raises(dp.OpenDPException, match=re.escape('The key-set of {col("B")} is private')):
        agg_query.release()


# Make key set invariant:
def test_with_max_length_and_invariant_keys():
    pl = pytest.importorskip("polars")
    
    context_w_invariant_keys = dp.Context.compositor(
        data=pl.LazyFrame({"A": [1, 2, 3, 4], "B": [5, 6, 7, 8]}),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1),
        split_evenly_over=2,
        margins=[
            dp.polars.Margin(
                by=[],
                max_length=1
            ),
            dp.polars.Margin(
                by=['B'],
                max_length=1,
                invariant='keys',
            ),
        ],
    )

    config = pl.col("A").dp.mean((0, 10))

    plain_query = context_w_invariant_keys.query().select(config)
    plain_query.release()

    agg_query = context_w_invariant_keys.query().group_by(["B"]).agg(config)
    agg_query.release()