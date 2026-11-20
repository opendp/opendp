import io
from typing import cast

import pytest
import opendp.prelude as dp
from opendp.extras.polars import LazyFrameQuery, Margin

pl = pytest.importorskip("polars")


atom = dp.atom_domain(bounds=(0.0, 10.0), nan=False)


def test_deserialize_polars_plan_error():
    context = dp.Context.compositor(
        data=[1, 2, 3],  # It will expect a LazyFrame here.
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    with (
        pytest.warns(DeprecationWarning),
        pytest.raises(ValueError, match=r"'data' of context must be a LazyFrame"),
    ):
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
    serialized = dp.serialize(dp_domain)
    deserialized = dp.deserialize(serialized)

    for domain in [dp_domain, deserialized]:
        if isinstance(in_value, pl.LazyFrame):
            for value in [in_value, out_value]:
                with pytest.raises(
                    dp.OpenDPException, match="membership is unsupported"
                ):
                    domain.member(value)
        else:
            assert domain.member(in_value)
            assert not domain.member(out_value)


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
query = cast(LazyFrameQuery, context.query()).select(dp.len(signed=True))


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


def test_lazyframequery_deserialize_is_safe_entrypoint(monkeypatch):
    import io

    original = cast(LazyFrameQuery, context.query())
    raw = original.polars_plan.serialize()

    from opendp import _data

    sanitize = _data.sanitize_polars_plan
    checked = []

    def checked_sanitize(data, trusted_lib, trusted):
        assert trusted is False
        checked.append(data)
        return sanitize(data, trusted_lib, trusted)

    monkeypatch.setattr(_data, "sanitize_polars_plan", checked_sanitize)
    restored = original.deserialize(raw)

    assert checked == [raw]
    assert isinstance(restored, type(original))
    assert restored is not original
    assert restored.polars_plan is not original.polars_plan
    assert restored._query is original._query
    assert restored._query._d_in == context.d_in
    assert context.d_mids is not None
    assert restored._query._d_out == context.d_mids[0]

    # Polars supports deserializing directly from a stream, but the OpenDP
    # entry point intentionally accepts binary plan bytes only.
    with pytest.raises(TypeError):
        original.deserialize(io.BytesIO(raw))  # type: ignore[arg-type]

    # Bypass through Polars' state-restoration method must also be unavailable.
    with pytest.raises(AttributeError, match="LazyFrameQuery.deserialize"):
        original.__setstate__(raw)


def test_deprecated_context_deserialize_polars_plan():
    serialized = query.polars_plan.serialize()
    with pytest.warns(DeprecationWarning, match="context.query\\(\\).deserialize"):
        restored = context.deserialize_polars_plan(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
    assert restored._query._d_in == context.d_in
    assert context.d_mids is not None
    assert restored._query._d_out == context.d_mids[0]


def _identity_batches(frame):
    return frame


def _identity_element(value):
    return value


def test_serialized_plan_preflight_allows_opendp_query():
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    serialized = query.polars_plan.serialize()
    assert isinstance(sanitize_polars_plan(serialized, str(lib_path), False), bytes)
    restored = context.query().deserialize(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
    renamed = pl.LazyFrame({"value": [1]}).select(
        pl.col("value").alias("renamed_value")
    )
    assert isinstance(
        sanitize_polars_plan(renamed.serialize(), str(lib_path), False), bytes
    )


@pytest.mark.parametrize(
    "expression",
    [
        pytest.param(pl.col("value").alias("renamed"), id="Alias"),
        pytest.param(pl.col("value"), id="Column"),
        pytest.param(pl.lit(1), id="Literal"),
        pytest.param(pl.element(), id="Element"),
        pytest.param(pl.len(), id="Len"),
        pytest.param(pl.col("value").name.keep(), id="KeepName"),
        pytest.param(pl.col("value") + 1, id="BinaryExpr"),
        pytest.param(pl.when(pl.col("value") > 0).then(1).otherwise(0), id="Ternary"),
        pytest.param(pl.col("value").cast(pl.Int64, strict=True), id="Cast"),
        pytest.param(pl.col("value").filter(pl.col("value") > 0), id="Expr-Filter"),
        pytest.param(pl.col("value").sort_by("group"), id="SortBy"),
        pytest.param(pl.col("value").rank("dense").over("group"), id="Over"),
        pytest.param(
            pl.col("value").rank("ordinal").over("group", order_by="value"),
            id="Over-order-by",
        ),
        pytest.param(pl.col("value").fill_null(0), id="FillNull"),
        pytest.param(pl.col("value").fill_nan(0), id="FillNan"),
        pytest.param(pl.col("value").clip(0, 10), id="Clip"),
        pytest.param(pl.col("value").cut([0, 3]), id="Cut"),
        pytest.param(pl.col("value").drop_nulls(), id="DropNulls"),
        pytest.param(pl.col("value").drop_nans(), id="DropNans"),
        pytest.param(pl.col("value").to_physical(), id="ToPhysical"),
        pytest.param(pl.col("value").replace({1: 2}), id="Replace"),
        pytest.param(
            pl.col("value").replace_strict({1: 2}, default=0), id="ReplaceStrict"
        ),
        pytest.param(pl.col("value").null_count(), id="NullCount"),
        pytest.param(pl.struct("value", "group"), id="AsStruct"),
        pytest.param(pl.col("value").hash(seed=1), id="Hash"),
        pytest.param(pl.col("value").rank("dense"), id="Rank"),
        pytest.param(pl.col("value").reverse(), id="Reverse"),
        pytest.param(pl.col("value").shuffle(seed=3), id="Random"),
        pytest.param(pl.col("value").abs(), id="Abs"),
        pytest.param(-pl.col("value"), id="Negate"),
        pytest.param(pl.col("value").floor(), id="Floor"),
        pytest.param(pl.col("value").ceil(), id="Ceil"),
        pytest.param(pl.col("value").round(1), id="Round"),
        pytest.param(pl.col("value").pow(2), id="Pow"),
        pytest.param(pl.col("value").is_null(), id="IsNull"),
        pytest.param(pl.col("value").is_not_null(), id="IsNotNull"),
        pytest.param(pl.col("value").is_nan(), id="IsNan"),
        pytest.param(pl.col("value").is_not_nan(), id="IsNotNan"),
        pytest.param(pl.col("value").is_finite(), id="IsFinite"),
        pytest.param(pl.col("value").is_infinite(), id="IsInfinite"),
        pytest.param(pl.col("value").is_between(0, 3), id="IsBetween"),
        pytest.param(pl.col("value").is_in([1, 2]), id="IsIn"),
        pytest.param(~(pl.col("value") > 0), id="Not"),
        pytest.param((pl.col("value") > 0).any(), id="Any"),
        pytest.param((pl.col("value") > 0).all(), id="All"),
        pytest.param(
            pl.any_horizontal(pl.col("value") > 0, pl.col("value") < 3),
            id="AnyHorizontal",
        ),
        pytest.param(
            pl.all_horizontal(pl.col("value") > 0, pl.col("value") < 3),
            id="AllHorizontal",
        ),
        pytest.param(pl.int_range(0, pl.len()), id="IntRange"),
        pytest.param(pl.int_ranges(0, pl.col("value").cast(pl.Int64)), id="IntRanges"),
        pytest.param(pl.col("value").count(), id="Agg-Count"),
        pytest.param(pl.col("value").n_unique(), id="Agg-NUnique"),
        pytest.param(pl.col("value").sum(), id="Agg-Sum"),
        pytest.param(pl.col("value").min(), id="Agg-Min"),
        pytest.param(pl.col("value").max(), id="Agg-Max"),
        pytest.param(pl.col("value").mean(), id="Agg-Mean"),
        pytest.param(pl.col("value").median(), id="Agg-Median"),
        pytest.param(pl.col("value").first(), id="Agg-First"),
        pytest.param(pl.col("value").last(), id="Agg-Last"),
        pytest.param(pl.col("value").quantile(0.5), id="Agg-Quantile"),
        pytest.param(pl.col("value").std(), id="Agg-Std"),
        pytest.param(pl.col("value").var(), id="Agg-Var"),
    ],
)
def test_preflight_accepts_core_analytical_expressions(expression):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    frame = pl.LazyFrame({"value": [1.0, 2.0], "group": ["a", "a"]})
    serialized = frame.select(expression).serialize()
    sanitized = sanitize_polars_plan(serialized, str(lib_path), False)
    assert isinstance(sanitized, bytes)
    assert isinstance(pl.LazyFrame.deserialize(io.BytesIO(sanitized)), pl.LazyFrame)


def test_preflight_accepts_core_plans():
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    frame = pl.LazyFrame({"value": [1.0, 2.0], "group": ["a", "a"]})
    other = pl.LazyFrame({"group": ["a"]})
    plans = [
        frame.filter(pl.col("value") > 0),
        frame.with_columns(pl.col("value").fill_nan(0)),
        frame.group_by("group").agg(pl.col("value").sum()),
        frame.join(other, on="group", how="left"),
    ]
    for plan in plans:
        assert isinstance(
            sanitize_polars_plan(plan.serialize(), str(lib_path), False), bytes
        )


@pytest.mark.parametrize(
    "plan",
    [
        pl.LazyFrame({"value": [1, 2]}).sort("value"),
        pl.LazyFrame({"value": [1, 2]}).select(pl.col("value").diff()),
    ],
    ids=["sort_plan", "unsupported_function"],
)
def test_preflight_rejects_unlisted_operations(plan):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    with pytest.raises(dp.OpenDPException, match="unsupported .* operation"):
        sanitize_polars_plan(plan.serialize(), str(lib_path), False)


@pytest.mark.parametrize(
    "kind",
    ["plan_callback", "expression_callback", "group_callback", "rename_callback"],
)
def test_serialized_plan_preflight_rejects_python_udfs(monkeypatch, kind):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    source = pl.LazyFrame({"value": [1]})
    if kind == "plan_callback":
        malicious = source.map_batches(_identity_batches, schema={"value": pl.Int64})
    elif kind == "expression_callback":
        malicious = source.select(
            pl.col("value").map_elements(_identity_element, return_dtype=pl.Int64)
        )
    elif kind == "group_callback":
        malicious = source.group_by("value").map_groups(
            _identity_batches, schema={"value": pl.Int64}
        )
    else:
        malicious = source.select(pl.col("value").name.map(_identity_element))
    serialized = malicious.serialize()
    with pytest.raises(dp.OpenDPException):
        sanitize_polars_plan(serialized, str(lib_path), False)

    # Construct the server query before patching Polars' own loader.
    initial_query = context.query()

    # The security preflight must fail before Python calls the Polars loader.
    def forbidden_deserialize(*args, **kwargs):
        pytest.fail("Polars deserialized an untrusted Python callback")

    monkeypatch.setattr(pl.LazyFrame, "deserialize", forbidden_deserialize)
    with pytest.raises(dp.OpenDPException):
        initial_query.deserialize(serialized)


def test_deserialize_reports_polars_version_mismatch():
    serialized = bytearray(query.polars_plan.serialize())
    serialized[len(b"DSL_VERSION")] ^= 1
    with pytest.raises(dp.OpenDPException, match="Use compatible Polars versions"):
        context.query().deserialize(serialized)


@pytest.mark.parametrize("format", ["parquet", "csv"])
def test_untrusted_deserialize_rejects_file_scans(tmp_path, monkeypatch, format):
    keys = tmp_path / f"keys.{format}"
    frame = pl.DataFrame({"age": [1, 2, 3]})
    if format == "parquet":
        frame.write_parquet(keys)
        scan = pl.scan_parquet(keys)
    else:
        frame.write_csv(keys)
        scan = pl.scan_csv(keys)

    plan = (
        cast(LazyFrameQuery, context.query())
        .group_by("age")
        .agg(dp.len(signed=True))
        .with_keys(scan)
    )

    initial_query = context.query()

    def reject_polars(*args, **kwargs):
        pytest.fail("Polars deserialized a file scan before preflight")

    monkeypatch.setattr(pl.LazyFrame, "deserialize", reject_polars)
    with pytest.raises(dp.OpenDPException, match="unsupported plan operation: Scan"):
        initial_query.deserialize(plan.polars_plan.serialize())


def test_untrusted_deserialize_allows_in_memory_public_keys():
    plan = (
        cast(LazyFrameQuery, context.query())
        .group_by("age")
        .agg(dp.len(signed=True))
        .with_keys(pl.LazyFrame({"age": [1, 2, 3]}))
    )
    restored = context.query().deserialize(plan.polars_plan.serialize())
    assert isinstance(restored.polars_plan, pl.LazyFrame)


def test_trusted_local_plan_can_scan_file(tmp_path, monkeypatch):
    from opendp._convert import _lazyframe_to_slice
    from opendp._data import slice_as_object

    source = tmp_path / "keys.csv"
    pl.DataFrame({"age": [1]}).write_csv(source)
    plan = pl.scan_csv(source)
    serialized = plan.serialize()

    # Normal local FFI conversion does not apply remote-plan restrictions.
    assert slice_as_object(_lazyframe_to_slice(plan), "LazyFrame") is not None

    with pytest.raises(dp.OpenDPException, match="unsupported plan operation: Scan"):
        context.query().deserialize(serialized)

    from opendp import _data

    sanitize = _data.sanitize_polars_plan

    def checked_preflight(data, trusted_lib, trusted):
        assert trusted is True
        return sanitize(data, trusted_lib, trusted)

    monkeypatch.setattr(_data, "sanitize_polars_plan", checked_preflight)
    restored = context.query().deserialize(serialized, trusted=True)
    assert isinstance(restored.polars_plan, pl.LazyFrame)


def test_deserialize_polars_plan_uses_same_bytes_after_validation(monkeypatch):
    from opendp import _data

    supplied = bytearray(query.polars_plan.serialize())
    sanitize = _data.sanitize_polars_plan

    def mutate_original(snapshot, trusted_lib, trusted):
        assert type(snapshot) is bytes
        rewritten = sanitize(snapshot, trusted_lib, trusted)
        supplied[:] = b"\x00" * len(supplied)
        return rewritten

    monkeypatch.setattr(_data, "sanitize_polars_plan", mutate_original)
    restored = context.query().deserialize(supplied)
    assert isinstance(restored.polars_plan, pl.LazyFrame)


def _fake_plugin(tmp_path, symbol="noise", *, args=None, kwargs=None, filename=None):
    from polars.plugins import register_plugin_function  # type: ignore[import-not-found]
    from opendp._lib import lib_path

    path = tmp_path / (filename or lib_path.name)
    path.write_bytes(b"not a shared library")
    return path, register_plugin_function(
        plugin_path=path,
        function_name=symbol,
        args=[pl.col("value")] if args is None else args,
        kwargs=kwargs,
        is_elementwise=True,
    )


@pytest.mark.parametrize(
    "symbol",
    [
        "noise",
        "dp_len",
        "dp_count",
        "dp_null_count",
        "dp_n_unique",
        "dp_frame_len",
        "dp_sum",
        "dp_mean",
        "dp_quantile",
        "dp_median",
        "noisy_max",
        "permute_plugin",
    ],
)
def test_preflight_accepts_all_native_plugin_symbols(tmp_path, symbol):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    fake_lib, expression = _fake_plugin(tmp_path, symbol)
    serialized = pl.LazyFrame({"value": [1]}).select(expression).serialize()
    rewritten = sanitize_polars_plan(serialized, str(lib_path), False)
    assert isinstance(rewritten, bytes)
    assert str(fake_lib).encode() not in rewritten


@pytest.mark.parametrize(
    ("symbol", "kwargs"),
    [
        ("unknown_symbol", None),
        ("noisy_max_plugin", None),
        ("noise", {"payload": [1]}),
    ],
    ids=["unrecognized", "executable", "pickled_kwargs"],
)
def test_preflight_rejects_unsupported_native_plugins(tmp_path, symbol, kwargs):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    _, expression = _fake_plugin(tmp_path, symbol, kwargs=kwargs)
    serialized = pl.LazyFrame({"value": [1]}).select(expression).serialize()
    with pytest.raises(
        dp.OpenDPException,
        match="unsupported native plugin|plugin keyword arguments",
    ):
        sanitize_polars_plan(serialized, str(lib_path), False)


@pytest.mark.parametrize("filename", ["libother.so", "libopendp_other.so"])
def test_trusted_preflight_preserves_third_party_plugins(tmp_path, filename):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    _, expression = _fake_plugin(tmp_path, filename=filename)
    serialized = pl.LazyFrame({"value": [1]}).select(expression).serialize()
    assert sanitize_polars_plan(serialized, str(lib_path), True) == serialized


@pytest.mark.parametrize("trusted", [False, True])
def test_serialized_plan_rebinds_untrusted_native_plugin(tmp_path, trusted):
    # The file is deliberately not a valid shared library. The remote library
    # path must never be passed to Polars' dynamic loader.
    _, plugin = _fake_plugin(tmp_path, args=[pl.col("value"), pl.lit(None)])
    serialized = pl.LazyFrame({"value": [1]}).select(plugin).serialize()
    restored = context.query().deserialize(serialized, trusted=trusted)
    assert isinstance(restored.polars_plan, pl.LazyFrame)


def test_trusted_deserialize_preserves_callback_when_rebinding_plugin(tmp_path):
    # Deserializing a trusted plan must preserve Python callbacks even when
    # rebinding an OpenDP plugin requires re-encoding the whole plan.
    _, plugin = _fake_plugin(tmp_path, args=[pl.col("value"), pl.lit(None)])
    plan = pl.LazyFrame({"value": [1, 2]}).with_columns(
        pl.col("value")
        .map_elements(_identity_element, return_dtype=pl.Int64)
        .alias("callback_value"),
        plugin.alias("noise"),
    )
    restored = cast(LazyFrameQuery, context.query()).deserialize(
        plan.serialize(), trusted=True
    )
    result = restored.polars_plan.select("callback_value").collect()
    assert result["callback_value"].to_list() == [1, 2]
