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


def test_lazyframequery_deserialize_is_safe_entrypoint(monkeypatch):
    import io

    original = context.query()
    raw = original.polars_plan.serialize()

    from opendp import _data

    sanitize = _data.sanitize_polars_plan
    checked = []

    def checked_sanitize(data, trusted_lib, allowed_files):
        checked.append(data)
        assert allowed_files == []
        return sanitize(data, trusted_lib, allowed_files)

    monkeypatch.setattr(_data, "sanitize_polars_plan", checked_sanitize)
    restored = original.deserialize(raw)

    assert checked == [raw]
    assert isinstance(restored, type(original))
    assert restored is not original
    assert restored.polars_plan is not original.polars_plan
    assert restored._query is original._query
    assert restored._query._d_in == context.d_in
    assert restored._query._d_out == context.d_mids[0]

    # Polars supports deserializing directly from a stream, but the OpenDP
    # entry point intentionally accepts binary plan bytes only.
    with pytest.raises(TypeError):
        original.deserialize(io.BytesIO(raw))

    # Bypass through Polars' state-restoration method must also be unavailable.
    with pytest.raises(AttributeError, match="LazyFrameQuery.deserialize"):
        original.__setstate__(raw)


def test_deprecated_context_deserialize_polars_plan():
    serialized = query.polars_plan.serialize()
    with pytest.warns(DeprecationWarning, match="context.query\\(\\).deserialize"):
        restored = context.deserialize_polars_plan(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
    assert restored._query._d_in == context.d_in
    assert restored._query._d_out == context.d_mids[0]


def _identity_batches(frame):
    return frame


def _identity_element(value):
    return value


def test_serialized_plan_preflight_allows_opendp_query():
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    serialized = query.polars_plan.serialize()
    assert isinstance(sanitize_polars_plan(serialized, str(lib_path)), bytes)
    restored = context.query().deserialize(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)
    renamed = pl.LazyFrame({"value": [1]}).select(
        pl.col("value").name.prefix("renamed_")
    )
    assert isinstance(sanitize_polars_plan(renamed.serialize(), str(lib_path)), bytes)


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
        sanitize_polars_plan(serialized, str(lib_path))

    # The security preflight must fail before Python calls the Polars loader.
    def forbidden_deserialize(*args, **kwargs):
        pytest.fail("Polars deserialized an untrusted Python callback")

    monkeypatch.setattr(pl.LazyFrame, "deserialize", forbidden_deserialize)
    with pytest.raises(dp.OpenDPException):
        context.query().deserialize(serialized)


def test_serialized_plan_preflight_rejects_malformed_bytes():
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path

    for data in [b"", b"not a Polars plan", b"DSL_VERSION"]:
        with pytest.raises(dp.OpenDPException):
            sanitize_polars_plan(data, str(lib_path))


def test_deserialize_reports_expected_polars_version():
    from opendp.mod import _EXPECTED_POLARS_VERSION

    serialized = bytearray(query.polars_plan.serialize())
    serialized[len(b"DSL_VERSION")] ^= 1
    with pytest.raises(ValueError, match=f"Expected Polars {_EXPECTED_POLARS_VERSION}"):
        context.query().deserialize(serialized)


def test_deserialize_file_allowlist(tmp_path, monkeypatch):
    keys = tmp_path / "keys.parquet"
    other = tmp_path / "other.parquet"
    pl.DataFrame({"age": [1, 2, 3]}).write_parquet(keys)
    pl.DataFrame({"age": [4]}).write_parquet(other)

    plan = (
        context.query()
        .group_by("age")
        .agg(dp.len(signed=True))
        .with_keys(pl.scan_parquet(keys))
    )
    serialized = plan.polars_plan.serialize()

    real_deserialize = pl.LazyFrame.deserialize

    def reject_polars(*args, **kwargs):
        pytest.fail("Polars deserialization ran before file authorization")

    monkeypatch.setattr(pl.LazyFrame, "deserialize", reject_polars)
    with pytest.raises(dp.OpenDPException, match="file scans are not allowed"):
        context.query().deserialize(serialized)
    with pytest.raises(dp.OpenDPException, match="file scan path is not allowed"):
        context.query().deserialize(serialized, allowed_files=[other])

    monkeypatch.setattr(pl.LazyFrame, "deserialize", real_deserialize)
    restored = context.query().deserialize(serialized, allowed_files=[keys])
    assert isinstance(restored.polars_plan, pl.LazyFrame)

    with pytest.raises(ValueError, match="Glob characters"):
        context.query().deserialize(serialized, allowed_files=[tmp_path / "*.parquet"])


def test_deserialize_rejects_non_parquet_scan(tmp_path):
    keys = tmp_path / "keys.csv"
    pl.DataFrame({"age": [1]}).write_csv(keys)
    serialized = pl.scan_csv(keys).serialize()
    with pytest.raises(dp.OpenDPException, match="only local Parquet"):
        context.query().deserialize(serialized, allowed_files=[keys])


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
    sanitize = _data.sanitize_polars_plan

    def mutate_original(snapshot, trusted_lib, allowed_files):
        assert type(snapshot) is bytes
        rewritten = sanitize(snapshot, trusted_lib, allowed_files)
        supplied[:] = b"\x00" * len(supplied)
        return rewritten

    monkeypatch.setattr(_data, "sanitize_polars_plan", mutate_original)
    restored = context.query().deserialize(supplied)
    assert isinstance(restored.polars_plan, pl.LazyFrame)


def test_serialized_plan_rebinds_untrusted_native_plugin(tmp_path):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path
    from polars.plugins import register_plugin_function

    # The file is deliberately not a valid shared library. The remote library
    # path must never be passed to Polars' dynamic loader.
    fake_lib = tmp_path / "spoof_opendp.so"
    fake_lib.write_bytes(b"not a shared library")
    plugin = register_plugin_function(
        plugin_path=fake_lib,
        function_name="noise",
        args=[pl.col("value"), pl.lit(None)],
        is_elementwise=True,
    )
    serialized = pl.LazyFrame({"value": [1]}).select(plugin).serialize()
    sanitized = sanitize_polars_plan(serialized, str(lib_path))
    assert str(fake_lib).encode() in serialized
    assert str(fake_lib).encode() not in sanitized
    assert str(lib_path).encode() in sanitized
    restored = context.query().deserialize(serialized)
    assert isinstance(restored.polars_plan, pl.LazyFrame)


@pytest.mark.parametrize(
    "kwargs,symbol",
    [
        ({"injected": 1}, "noise"),
        (None, "arbitrary_symbol"),
    ],
)
def test_serialized_plan_rejects_untrusted_native_plugin(tmp_path, kwargs, symbol):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path
    from polars.plugins import register_plugin_function

    fake_lib = tmp_path / "spoof_opendp.so"
    fake_lib.write_bytes(b"not a shared library")
    plugin = register_plugin_function(
        plugin_path=fake_lib,
        function_name=symbol,
        args=[pl.col("value"), pl.lit(None)],
        kwargs=kwargs,
        is_elementwise=True,
    )
    serialized = pl.LazyFrame({"value": [1]}).select(plugin).serialize()
    with pytest.raises(dp.OpenDPException):
        sanitize_polars_plan(serialized, str(lib_path))


def test_serialized_plan_does_not_restrict_client_plugin_flags(tmp_path):
    from opendp._data import sanitize_polars_plan
    from opendp._lib import lib_path
    from polars.plugins import register_plugin_function

    fake_lib = tmp_path / "spoof_opendp.so"
    fake_lib.write_bytes(b"not a shared library")
    # OpenDP rewrites the executable plugin with its canonical options.
    plugin = register_plugin_function(
        plugin_path=fake_lib,
        function_name="noise",
        args=[pl.col("value"), pl.lit(None)],
        is_elementwise=False,
    )
    serialized = pl.LazyFrame({"value": [1]}).select(plugin).serialize()
    sanitized = sanitize_polars_plan(serialized, str(lib_path))
    assert str(fake_lib).encode() not in sanitized
