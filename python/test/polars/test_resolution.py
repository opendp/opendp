import opendp.prelude as dp
from opendp.extras import polars as dp_polars
import pytest

pl = pytest.importorskip("polars")


def test_adapter_runs_after_calibration_but_not_summary(monkeypatch):
    context = dp.Context.compositor(
        pl.LazyFrame({"value": [1, 2, 3]}),
        dp.unit_of(contributions=1),
        dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
        margins=[dp.polars.Margin(by=["value"], max_length=3, invariant="keys")],
    )
    constructed, adapted = [], []
    make_private = dp_polars.make_private_lazyframe

    def count_construction(*args, **kwargs):
        constructed.append(kwargs["global_scale"])
        return make_private(*args, **kwargs)

    def adapter(native):
        adapted.append(native)
        return native

    monkeypatch.setattr(dp_polars, "make_private_lazyframe", count_construction)
    assert isinstance(context.queryable, dp.Queryable)
    adapted_context = dp.Context(
        context.accountant,
        context.queryable,
        context.d_in,
        d_mids=context.d_mids,
        measurement_adapter=adapter,
    )
    query = (
        adapted_context.query()
        .select("value")
        .group_by("value")
        .agg(dp.len(signed=False).alias("len"))
    )
    assert query.summarize().height == 1
    assert len(constructed) > 1 and adapted == []
    resolved = query.resolve()
    assert adapted == [resolved]
    assert not hasattr(resolved, "_polars_measurement")
    assert query.summarize().height == 1
    assert adapted == [resolved]
    assert query.release().collect().shape == (3, 2)
    assert len(adapted) == 2
    keyed = query.with_keys(pl.DataFrame({"value": [1, 2, 3]}))
    resolved_keyed = keyed.resolve()
    assert len(adapted) == 3 and adapted[-1] is resolved_keyed
