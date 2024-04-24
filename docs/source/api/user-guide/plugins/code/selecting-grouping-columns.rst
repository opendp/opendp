:orphan:

# user-defined-functions
>>> import opendp.prelude as dp
>>> import pandas as pd
>>> import faker
>>> import random

>>> def make_grouping_cols_score(candidates, min_bin_contributions):
...     r"""Create a transformation that assesses the utility of each candidate in `candidates`.
...     Try to select a set of columns to group by that will maximize the number of columns selected,
...     but won't result in bins that are too sparse when used with bin censoring.
...     A rough heuristic is to score each candidate grouping set
...     by the number of bins with at least `min_bin_contributions`.
...     """
...     dp.assert_features("contrib")
...
...     # define a function that scores an individual candidate
...     def score(x: pd.DataFrame, c):
...         return (x.groupby(list(c)).size() >= min_bin_contributions).sum().astype(float)
...
...     # define a stable transformation that aggregates a dataframe into a vector of scores
...     #   (one score per candidate)
...     return dp.t.make_user_transformation(
...         # create a new domain (the set of all pandas dataframes)
...         input_domain=dp.user_domain(
...             "PandasDomain", member=lambda x: isinstance(x, pd.DataFrame)
...         ),
...         input_metric=dp.symmetric_distance(),
...         output_domain=dp.vector_domain(dp.atom_domain(T=float)),
...         output_metric=dp.linf_distance(T=float, monotonic=True),
...         function=lambda x: [score(x, c) for c in candidates],
...         # the mechanism is 1-stable under the l-infinity distance,
...         #    as the addition or removal of any one record changes each score by at most one
...         stability_map=lambda d_in: float(d_in),
...     )

>>> def make_select_grouping_cols(candidates, min_bin_size, scale):
...     """Create a measurement that selects a set of grouping columns from `candidates`."""
...     return (
...         make_grouping_cols_score(candidates, min_bin_size)
...         >> dp.m.then_report_noisy_max_gumbel(scale, optimize="max")
...         >> (lambda idx: candidates[idx])
...     )

# /user-defined-functions

# dp-mechanism
>>> candidates = [
...     ("date", "merch_category", "transaction_type"),
...     ("date", "merchant_postal_code"),
...     ("date", "merchant_postal_code", "merch_category"),
...     ("date", "merchant_postal_code", "merch_category", "transaction_type"),
... ]


>>> dp.enable_features("honest-but-curious", "contrib")
>>> m_select_gcols = make_select_grouping_cols(
...     candidates=candidates,
...     min_bin_size=89,
...     scale=10.0,
... )

>>> print("ε =", m_select_gcols.map(d_in=1))
ε = 0.1

# /dp-mechanism

# dp-release
>>> fake = faker.Faker()
>>> n_records = 10_000
>>> data = pd.DataFrame(
...     {
...         "date": [fake.date() for _ in range(n_records)],
...         "merch_category": [random.choice([1, 2, 3]) for _ in range(n_records)],
...         "transaction_type": [random.choice([1, 2, 3]) for _ in range(n_records)],
...         "merchant_postal_code": [fake.zipcode() for _ in range(n_records)],
...     }
... )

>>> dp_selected_grouping_columns = m_select_gcols(data)
>>> print(dp_selected_grouping_columns)
(...)

# /dp-release