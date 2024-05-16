:orphan:

>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")
>>> import polars as pl
>>> import urllib.request

>>> data_url = "https://raw.githubusercontent.com/opendp/opendp/sydney/teacher_survey.csv"
>>> df = pl.read_csv(urllib.request.urlopen(data_url), has_header=False)
>>> age_df = df.rename({'column_3': 'age'}).select('age')

>>> plan = pl.col('age').mean()
>>> dp_mean = dp.m.make_private_lazyframe(
...   input_domain=dp.lazyframe_domain([dp.series_domain("age", dp.atom_domain(T=dp.i32))]),
...   input_metric=dp.symmetric_distance(),
...   output_measure=dp.max_divergence(T=int),
...   lazyframe=plan,
...   global_scale=1.0)

>>> lf = pl.LazyFrame(age_df)
>>> dp_mean(lf)
???