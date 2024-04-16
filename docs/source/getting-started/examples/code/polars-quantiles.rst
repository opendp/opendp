>>> import opendp.prelude as dp
>>> import polars as pl


>>> dp.enable_features("contrib")


>>> lf = pl.LazyFrame([
...     pl.Series("A", [1.0] * 50, dtype=pl.Float64),
...     pl.Series("B", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
...     pl.Series("C", ["1"] * 49 + [None], dtype=pl.String),
...     pl.Series("D", [2] * 50, dtype=pl.Int32),
... ])


>>> # specify domain descriptors of the lazyframe (without the data)
... lf_domain = dp.lazyframe_domain([
...     dp.series_domain("A", dp.atom_domain(T=dp.f64)),
...     dp.series_domain("B", dp.atom_domain(T=dp.i32)),
...     dp.series_domain("C", dp.option_domain(dp.atom_domain(T=dp.String))),
...     dp.series_domain("D", dp.atom_domain(T=dp.i32)),
... ])

>>> # specify properties of the data when grouped by "B"
>>> lf_domain = dp.with_margin(lf_domain, by=["B"], public_info="keys", max_partition_length=50)

>>> # USER STARTS HERE
>>> candidates = list(range(0, 110, 10))

>>> # lf domain is a domain.
>>> # we need a way to create a starter lazyframe to manipulate with the Polars API
>>> # proposed_plan = lf_domain.new_empty_lazyframe().group_by("B").agg([
>>> #     pl.col("A").dp.sum(bounds=(0.0, 1.0), scale=2.),
>>> #     pl.col("D").dp.quantile(candidates, alpha=.75, scale=1.),
>>> # ])



>>> def seed(schema):
...     return pl.DataFrame(None, schema, orient="row").lazy()  # type: ignore[attr-defined]

>>> proposed_plan = seed(lf.schema).group_by("B").agg([
...     pl.col("A").dp.sum(bounds=(0.0, 1.0), scale=2.),
...     pl.col("D").dp.quantile(candidates, alpha=.75, scale=1.),
... ])

# Context API to look like this:
# context.query().group_by("B").agg([
#     pl.col("A").dp.sum(bounds=(0.0, 1.0), scale=2.),
#     pl.col("D").dp.quantile(candidates, alpha=.75, scale=1.),
# ]).release()


>>> # IN SERVER
>>> m_lf = dp.m.make_private_lazyframe(
...     input_domain=lf_domain, 
...     input_metric=dp.symmetric_distance(), 
...     output_measure=dp.max_divergence(T=float), 
...     lazyframe=proposed_plan, 
...     param=1.
... )

>>> df_release = m_lf(lf).collect()

>>> print(df_release) # doctest: +ELLIPSIS
shape: (5, 3)
┌─────┬───────────┬─────┐
│ B   ┆ A         ┆ D   │
│ --- ┆ ---       ┆ --- │
│ i32 ┆ f64       ┆ i64 │
╞═════╪═══════════╪═════╡
...

>>> # WHILE FUNCTIONALITY IS NOT YET SUPPORTED: 
>>> #    additional preprocessing (outside the framework) can be done here:
>>> # df_release = m_lf(lf.with_columns([pl.col("D").fill_nan(23.)])).collect()

>>> # will have a pre-release available later this week:
>>> # pip install opendp[polars] --pre