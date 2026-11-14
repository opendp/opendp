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


def _identity_batches(frame):
    return frame


def _identity_element(value):
    return value


def test_serialized_plan_preflight_allows_opendp_query():
    from opendp._data import validate_polars_plan

    serialized = query.polars_plan.serialize()
    assert validate_polars_plan(serialized) is True
    restored = context.deserialize_polars_plan(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
    renamed = pl.LazyFrame({"value": [1]}).select(
        pl.col("value").name.prefix("renamed_")
    )
    assert validate_polars_plan(renamed.serialize()) is True


@pytest.mark.parametrize(
    "kind", ["plan_callback", "expression_callback", "group_callback", "rename_callback"]
)
def test_serialized_plan_preflight_rejects_python_udfs(monkeypatch, kind):
    from opendp._data import validate_polars_plan

    source = pl.LazyFrame({"value": [1]})
    if kind == "plan_callback":
        malicious = source.map_batches(
            _identity_batches, schema={"value": pl.Int64}
        )
    elif kind == "expression_callback":
        malicious = source.select(
            pl.col("value").map_elements(
                _identity_element, return_dtype=pl.Int64
            )
        )
    elif kind == "group_callback":
        malicious = source.group_by("value").map_groups(
            _identity_batches, schema={"value": pl.Int64}
        )
    else:
        malicious = source.select(pl.col("value").name.map(_identity_element))
    serialized = malicious.serialize()
    with pytest.raises(dp.OpenDPException):
        validate_polars_plan(serialized)

    # The security preflight must fail before Python calls the Polars loader.
    def forbidden_deserialize(*args, **kwargs):
        pytest.fail("Polars deserialized an untrusted Python callback")

    monkeypatch.setattr(pl.LazyFrame, "deserialize", forbidden_deserialize)
    with pytest.raises(dp.OpenDPException):
        context.deserialize_polars_plan(serialized)


def test_serialized_plan_preflight_rejects_malformed_bytes():
    from opendp._data import validate_polars_plan

    for data in [b"", b"not a Polars plan", b"DSL_VERSION"]:
        with pytest.raises(dp.OpenDPException):
            validate_polars_plan(data)



def test_ffi_plan_loader_preflights_callbacks():
    from opendp._convert import _lazyframe_to_slice
    from opendp._data import slice_as_object

    source = pl.LazyFrame({"value": [1]})
    malicious = source.map_batches(_identity_batches, schema={"value": pl.Int64})
    with pytest.raises(dp.OpenDPException):
        slice_as_object(_lazyframe_to_slice(malicious), "LazyFrame")



def test_deserialize_polars_plan_uses_same_bytes_after_validation(monkeypatch):
    from opendp import _data

    supplied = bytearray(query.polars_plan.serialize())
    validate = _data.validate_polars_plan

    def mutate_original(snapshot):
        assert type(snapshot) is bytes
        assert validate(snapshot) is True
        supplied[:] = b"\x00" * len(supplied)
        return True

    monkeypatch.setattr(_data, "validate_polars_plan", mutate_original)
    restored = context.deserialize_polars_plan(supplied)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
