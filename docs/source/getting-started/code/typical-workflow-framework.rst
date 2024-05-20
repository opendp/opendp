:orphan:

# unit-of-privacy
>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")

>>> d_in = 1 # neighboring data set distance is at most d_in...
>>> input_metric = dp.symmetric_distance() # ...in terms of additions/removals

# /unit-of-privacy


# privacy-loss
>>> d_out = 1. # output distributions have distance at most d_out (ε)...
>>> privacy_measure = dp.max_divergence(T=float) # ...in terms of pure-DP

# /privacy-loss


# public-info
>>> col_names = [
...    "name", "sex", "age", "maritalStatus", "hasChildren", "highestEducationLevel", 
...    "sourceOfStress", "smoker", "optimism", "lifeSatisfaction", "selfEsteem"
... ]

# /public-info


# mediate
>>> import urllib.request
>>> data_url = "https://raw.githubusercontent.com/opendp/opendp/sydney/teacher_survey.csv"
>>> if data is None:
...     with urllib.request.urlopen(data_url) as data_req:
...         data = data_req.read().decode('utf-8')

>>> m_sc = dp.c.make_sequential_composition(
...     # data set is a single string, with rows separated by linebreaks
...     input_domain=dp.atom_domain(T=str),
...     input_metric=input_metric,
...     output_measure=privacy_measure,
...     d_in=d_in,
...     d_mids=[d_out / 3] * 3,
... )

>>> # Call measurement with data to create a queryable:
>>> qbl_sc = m_sc(data)

# /mediate


# count
>>> count_transformation = (
...     dp.t.make_split_dataframe(",", col_names=col_names)
...     >> dp.t.make_select_column("age", str)
...     >> dp.t.then_count()
... )

>>> count_sensitivity = count_transformation.map(d_in)
>>> count_sensitivity
1

>>> count_measurement = dp.binary_search_chain(
...     lambda scale: count_transformation >> dp.m.then_laplace(scale), d_in, d_out / 3
... )
>>> dp_count = qbl_sc(count_measurement)

# /count


# mean
>>> mean_transformation = (
...     dp.t.make_split_dataframe(",", col_names=col_names) >>
...     dp.t.make_select_column("age", str) >>
...     dp.t.then_cast_default(float) >>
...     dp.t.then_clamp((18.0, 70.0)) >>  # a best-guess based on public information
...     dp.t.then_resize(size=dp_count, constant=42.0) >>
...     dp.t.then_mean()
... )

>>> mean_measurement = dp.binary_search_chain(
...     lambda scale: mean_transformation >> dp.m.then_laplace(scale), d_in, d_out / 3
... )

>>> dp_mean = qbl_sc(mean_measurement)

# /mean
