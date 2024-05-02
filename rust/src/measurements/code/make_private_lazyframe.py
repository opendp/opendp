import opendp.prelude as dp
import polars as pl
dp.enable_features("contrib")


lf_domain = dp.lazyframe_domain([
    dp.series_domain("grade", dp.atom_domain(T=dp.i32)),
    dp.series_domain("pet_count", dp.atom_domain(T=dp.i32))])

lf_domain_with_margin = dp.with_margin(
    lf_domain,
    by=["grade"],
    public_info="keys",
    max_partition_length=50)

plan = (
    pl.LazyFrame(schema={'grade': pl.Int32, 'pet_count': pl.Int32})
    .group_by("grade")
    .agg(pl.col("pet_count").dp.sum((0, 10), scale=1.0)))

dp_sum_pets_by_grade = dp.m.make_private_lazyframe(
    input_domain=lf_domain_with_margin,
    input_metric=dp.symmetric_distance(),
    output_measure=dp.max_divergence(T=float),
    lazyframe=plan,
    global_scale=1.0)

df = pl.from_records(
    [
        [0, 0], # No kindergarteners with pets.
        [0, 0],
        [0, 0],
        [1, 1], # Each first grader has 1 pet.
        [1, 1],
        [1, 1],
        [2, 1], # One second grader has chickens!
        [2, 1],
        [2, 9]
    ],
    schema=['grade', 'pet_count'])
lf = pl.LazyFrame(df)
results = dp_sum_pets_by_grade(lf).sort("grade").collect()
print(results) # doctest: +ELLIPSIS