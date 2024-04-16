:orphan:

# init
>>> import opendp.prelude as dp
>>> import polars as pl
>>> dp.enable_features("contrib")

# /init

# init-domain
>>> lf_domain = dp.lazyframe_domain([
...     dp.series_domain("A", dp.atom_domain(T=dp.f64)),
...     dp.series_domain("B", dp.atom_domain(T=dp.i32)),
...     dp.series_domain("D", dp.atom_domain(T=dp.i32)),
... ])

# /init-domain

# margin-domain
>>> lf_domain_with_margin = dp.with_margin(lf_domain, by=["B"], public_info="keys", max_partition_length=50)

# /margin-domain


# candidates
>>> candidates = [10 * i for i in range(11)]
>>> candidates
[0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100]

# /candidates

# plan
>>> schema_from_domain = { # TODO: Utility to extract this from domain
...     'A': pl.Float64,
...     'B': pl.Int32,
...     'D': pl.Int32
... }
>>> empty_lf = pl.DataFrame(None, schema_from_domain, orient="row").lazy()
>>> plan = empty_lf.group_by("B").agg([
...     pl.col("A").dp.sum(bounds=(0.0, 1.0), scale=2.),
...     pl.col("D").dp.quantile(candidates, alpha=.75, scale=1.),
... ])

>>> measurement = dp.m.make_private_lazyframe(
...     input_domain=lf_domain_with_margin, 
...     input_metric=dp.symmetric_distance(), 
...     output_measure=dp.max_divergence(T=float), 
...     lazyframe=plan, 
...     param=1.
... )

# /plan

# dp-release
>>> lf = pl.LazyFrame([
...     pl.Series("A", [1.0] * 50, dtype=pl.Float64),
...     pl.Series("B", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32),
...     pl.Series("D", [2] * 50, dtype=pl.Int32),
... ])

>>> release = measurement(lf).collect()

>>> print(release) # doctest: +ELLIPSIS
shape: (5, 3)
┌─────┬───────────┬─────┐
│ B   ┆ A         ┆ D   │
│ --- ┆ ---       ┆ --- │
│ i32 ┆ f64       ┆ i64 │
╞═════╪═══════════╪═════╡
...

# /dp-release