import pytest
import opendp.prelude as dp
from opendp.extras.polars import Margin

pl = pytest.importorskip("polars")


atom = dp.atom_domain(bounds=(0.0, 10.0), nan=False)


def test_deserialize_polars_plan_error():
    context = dp.Context.compositor(
        data=[1, 2, 3],  # It will expect a LazyFrame here.
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    with pytest.raises(ValueError, match=r"'data' of context must be a LazyFrame"):
        context.deserialize_polars_plan(pl.LazyFrame({}).serialize())


# member() will warn if instance is not even of the carrier type,
# but that behavior is tested elsewhere, and can be ignored here.
@pytest.mark.filterwarnings("ignore::UserWarning")
@pytest.mark.parametrize(
    "dp_domain,in_value,out_value",
    [
        (atom, 10, 100),
        # TODO: Might not be specifying categorical values correctly,
        # but shouldn't error, regardless.
        # https://github.com/opendp/opendp/issues/2264
        # (dp.categorical_domain(['A', 'B', 'C']),
        #  pl.lit("A", dtype=pl.Categorical),
        #  pl.lit("Z", dtype=pl.Categorical)
        # ),
        (
            dp.series_domain("name", atom),
            pl.Series("name", [1.0, 2.0, 3.0]),
            pl.Series("name", ["a", "b", "c"]),
        ),
        (
            dp.lazyframe_domain([dp.series_domain("A", atom)]),
            pl.LazyFrame({"A": [1.0, 2.0, 3.0]}),
            pl.LazyFrame({"A": ["a", "b", "c"]}),
        ),
    ],
    ids=lambda arg: str(arg),
)
def test_serializable_domain(dp_domain, in_value, out_value):
    assert dp_domain.member(in_value)
    assert not dp_domain.member(out_value)

    serialized = dp.serialize(dp_domain)
    deserialized = dp.deserialize(serialized)

    assert deserialized.member(in_value)
    assert not deserialized.member(out_value)


@pytest.mark.parametrize(
    "dp_measurement,value,output_type",
    [
        (dp.m.make_gaussian(atom, dp.absolute_distance(float), 1), 0, float),
    ],
    ids=lambda arg: str(arg),
)
def test_serializable_measurement(dp_measurement, value, output_type):
    assert isinstance(dp_measurement(value), output_type)


lf = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
lf_domain = dp.lazyframe_domain(
    [
        dp.series_domain("A", dp.atom_domain(T="i32")),
        dp.series_domain("B", dp.atom_domain(T=str)),
    ]
)
lf_domain_with_margin = dp.with_margin(lf_domain, Margin(by=[], max_length=1000))

context = dp.Context.compositor(
    data=pl.LazyFrame({"age": [1, 2, 3]}),
    privacy_unit=dp.unit_of(contributions=1),
    privacy_loss=dp.loss_of(epsilon=1.0),
    split_evenly_over=10,
)
query = context.query().select(dp.len(signed=True))


@pytest.mark.parametrize(
    "dp_obj",
    [
        lf_domain,
        lf_domain_with_margin,
        dp.m.make_private_lazyframe(
            lf_domain_with_margin,
            dp.symmetric_distance(),
            dp.max_divergence(),
            lf.select([dp.len(signed=True), pl.col("A").dp.sum((0, 1))]),
            global_scale=1.0,
        ),
        dp.m.make_private_expr(
            dp.wild_expr_domain([], dp.polars.Margin(by=[])),
            dp.l01inf_distance(dp.symmetric_distance()),
            dp.max_divergence(),
            dp.len(scale=1.0, signed=True),
        ),
    ],
    ids=lambda arg: str(arg),
)
def test_serializable_polars(dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    assert serialized == dp.serialize(deserialized)


@pytest.mark.parametrize(
    "dp_obj",
    [context, dp.Queryable("value", "query_type"), query],
    ids=lambda arg: str(arg),
)
def test_not_currently_serializable(dp_obj):
    with pytest.raises(Exception, match=r"OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)
