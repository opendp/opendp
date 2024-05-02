import opendp.prelude as dp
import polars as pl
dp.enable_features("contrib")

lf_domain = dp.lazyframe_domain([
    dp.series_domain("grades", dp.atom_domain(T=dp.f64)),
    dp.series_domain("classes", dp.atom_domain(T=dp.i32))])
lf = pl.LazyFrame([
    pl.Series("grades", [1.0] * 50, dtype=pl.Float64),
    pl.Series("classes", [1, 2, 3, 4, 5] * 10, dtype=pl.Int32)])

lf_domain = dp.with_margin(lf_domain, by=["classes"], public_info="keys", max_partition_length=50)


expr = pl.col("grades").dp.sum((1.0, 2.0), scale=0.0)
plan = pl.DataFrame(None, lf.schema, orient="row").lazy().group_by("classes").agg(expr)

m_lf = dp.m.make_private_lazyframe(
        lf_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan, 0.0
    )

df_exp = pl.DataFrame([
    pl.Series("classes", list(range(1, 6)), dtype=pl.Int32),
    pl.Series("grades", [10.0] * 5),
])
df_act = m_lf(lf).sort("classes").collect()

print('50?', lf.select(expr).collect()["grades"][0])