First, create a measurement with the Polars API:

>>> import opendp.prelude as dp
>>> import polars as pl
>>> dp.enable_features("contrib")
... 
>>> lf = pl.LazyFrame(schema={"A": pl.Int32, "B": pl.String})
>>> lf_domain = dp.lazyframe_domain([
...     dp.series_domain("A", dp.atom_domain(T="i32")), 
...     dp.series_domain("B", dp.atom_domain(T=str))
... ])
>>> lf_domain = dp.with_margin(lf_domain, by=[], max_partition_length=1000)
>>> meas = dp.m.make_private_lazyframe(
...     lf_domain,
...     dp.symmetric_distance(),
...     dp.max_divergence(),
...     lf.select([dp.len(), pl.col("A").dp.sum((0, 1))]),
...     global_scale=1.0
... )

This function extracts utility information about each aggregate in the resulting data frame:

>>> dp.summarize_polars_measurement(meas)
shape: (2, 4)
┌────────┬──────────────┬─────────────────┬───────┐
│ column ┆ aggregate    ┆ distribution    ┆ scale │
│ ---    ┆ ---          ┆ ---             ┆ ---   │
│ str    ┆ str          ┆ str             ┆ f64   │
╞════════╪══════════════╪═════════════════╪═══════╡
│ len    ┆ Frame Length ┆ Integer Laplace ┆ 1.0   │
│ A      ┆ Sum          ┆ Integer Laplace ┆ 1.0   │
└────────┴──────────────┴─────────────────┴───────┘

If you pass an alpha argument, then you also get accuracy estimates:

>>> dp.summarize_polars_measurement(meas, alpha=.05)
shape: (2, 5)
┌────────┬──────────────┬─────────────────┬───────┬──────────┐
│ column ┆ aggregate    ┆ distribution    ┆ scale ┆ accuracy │
│ ---    ┆ ---          ┆ ---             ┆ ---   ┆ ---      │
│ str    ┆ str          ┆ str             ┆ f64   ┆ f64      │
╞════════╪══════════════╪═════════════════╪═══════╪══════════╡
│ len    ┆ Frame Length ┆ Integer Laplace ┆ 1.0   ┆ 3.375618 │
│ A      ┆ Sum          ┆ Integer Laplace ┆ 1.0   ┆ 3.375618 │
└────────┴──────────────┴─────────────────┴───────┴──────────┘
