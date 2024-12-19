>>> dp.enable_features("contrib")
>>> import polars as pl

We'll imagine an elementary school is taking a pet census.
The private census data will have two columns: 

>>> lf_domain = dp.lazyframe_domain([
...     dp.series_domain("grade", dp.atom_domain(T=dp.i32)),
...     dp.series_domain("pet_count", dp.atom_domain(T=dp.i32))])

We also need to specify the column we'll be grouping by.

>>> lf_domain_with_margin = dp.with_margin(
...     lf_domain,
...     by=["grade"],
...     public_info="keys",
...     max_partition_length=50)

With that in place, we can plan the Polars computation, using the `dp` plugin. 

>>> plan = (
...     pl.LazyFrame(schema={'grade': pl.Int32, 'pet_count': pl.Int32})
...     .group_by("grade")
...     .agg(pl.col("pet_count").dp.sum((0, 10), scale=1.0)))

We now have all the pieces to make our measurement function using `make_private_lazyframe`:

>>> dp_sum_pets_by_grade = dp.m.make_private_lazyframe(
...     input_domain=lf_domain_with_margin,
...     input_metric=dp.symmetric_distance(),
...     output_measure=dp.max_divergence(),
...     lazyframe=plan,
...     global_scale=1.0)

It's only at this point that we need to introduce the private data.

>>> df = pl.from_records(
...     [
...         [0, 0], # No kindergarteners with pets.
...         [0, 0],
...         [0, 0],
...         [1, 1], # Each first grader has 1 pet.
...         [1, 1],
...         [1, 1],
...         [2, 1], # One second grader has chickens!
...         [2, 1],
...         [2, 9]
...     ],
...     schema=['grade', 'pet_count'], orient="row")
>>> lf = pl.LazyFrame(df)
>>> results = dp_sum_pets_by_grade(lf).collect()
>>> print(results.sort("grade")) # doctest: +ELLIPSIS
shape: (3, 2)
┌───────┬───────────┐
│ grade ┆ pet_count │
│ ---   ┆ ---       │
│ i64   ┆ i64       │
╞═══════╪═══════════╡
│ 0     ┆ ...       │
│ 1     ┆ ...       │
│ 2     ┆ ...       │
└───────┴───────────┘