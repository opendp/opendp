:orphan:

# init
>>> import opendp.prelude as dp
>>> import polars as pl
>>> dp.enable_features("contrib")

# /init

# init-domain
>>> lf_domain = dp.lazyframe_domain([
...     dp.series_domain("grouping-key", dp.atom_domain(T=dp.i32)),
...     dp.series_domain("twice-key", dp.atom_domain(T=dp.i32)),
...     dp.series_domain("ones", dp.atom_domain(T=dp.f64)),
... ])

# /init-domain

# margin-domain
>>> lf_domain_with_margin = dp.with_margin(
...     lf_domain,
...     by=["grouping-key"],
...     public_info="keys",
...     max_partition_length=50
... )

# /margin-domain

# plan
>>> schema_from_domain = { # TODO: Utility to extract this from domain
...     'grouping-key': pl.Int32,
...     'twice-key': pl.Int32,
...     'ones': pl.Float64,
... }
>>> empty_lf = pl.DataFrame(None, schema_from_domain, orient="row").lazy()
>>> plan = empty_lf.group_by("grouping-key").agg([
...     pl.col("ones").dp.sum(bounds=(0.0, 2.0), scale=2.),
...     pl.col("twice-key").dp.quantile([2, 4, 6, 8, 10], alpha=.75, scale=1.0),
... ])

# /plan

# measurement
>>> measurement = dp.m.make_private_lazyframe(
...     input_domain=lf_domain_with_margin, 
...     input_metric=dp.symmetric_distance(), 
...     output_measure=dp.max_divergence(T=float), 
...     lazyframe=plan, 
...     param=1.
... )

# /measurement

# dp-release
>>> private_lf = pl.LazyFrame([
...     pl.Series("grouping-key", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
...     pl.Series("twice-key", [2, 4, 6, 8, 10] * 10, dtype=pl.Int32),
...     pl.Series("ones", [1.0] * 50, dtype=pl.Float64),
... ])
>>> release = measurement(private_lf).collect().sort("grouping-key")
>>> print(release) # doctest: +ELLIPSIS
shape: (5, 3)
┌──────────────┬───────────┬───────────┐
│ grouping-key ┆ ones      ┆ twice-key │
│ ---          ┆ ---       ┆ ---       │
│ i32          ┆ f64       ┆ i64       │
╞══════════════╪═══════════╪═══════════╡
...

# /dp-release