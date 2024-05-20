:orphan:
# unit-of-privacy
>>> import opendp.prelude as dp
>>> dp.enable_features("contrib")

>>> privacy_unit = dp.unit_of(contributions=1)
>>> input_metric, d_in = privacy_unit

# /unit-of-privacy


# privacy-loss
>>> privacy_loss = dp.loss_of(epsilon=1.)
>>> privacy_measure, d_out = privacy_loss

# /privacy-loss


# public-info
>>> col_names = [
...    "name", "sex", "age", "maritalStatus", "hasChildren", "highestEducationLevel", 
...    "sourceOfStress", "smoker", "optimism", "lifeSatisfaction", "selfEsteem"
... ]

# /public-info


>>> data = 'age\n42\n' # Minimal data so doctest can run without hitting network.

# mediate
>>> import urllib.request
>>> data_url = "https://raw.githubusercontent.com/opendp/opendp/sydney/teacher_survey.csv"
>>> if data is None:
...     with urllib.request.urlopen(data_url) as data_req:
...         data = data_req.read().decode('utf-8')

>>> context = dp.Context.compositor(
...     data=data,
...     privacy_unit=privacy_unit,
...     privacy_loss=privacy_loss,
...     split_evenly_over=3
... )

# /mediate


# count
>>> count_query = (
...     context.query()
...     .split_dataframe(",", col_names=col_names)
...     .select_column("age", str) # temporary until OpenDP 0.10 (Polars dataframe)
...     .count()
...     .laplace()
... )

>>> scale = count_query.param()
>>> scale
3.0000000000000004

>>> accuracy = dp.discrete_laplacian_scale_to_accuracy(scale=scale, alpha=0.05)
>>> accuracy
9.445721638273584

>>> dp_count = count_query.release()
>>> interval = (dp_count - accuracy, dp_count + accuracy)

# /count


# mean
>>> mean_query = (
...     context.query()
...     .split_dataframe(",", col_names=col_names)
...     .select_column("age", str)
...     .cast_default(float)
...     .clamp((18.0, 70.0))  # a best-guess based on public information
...     # Explanation for `constant=42`:
...     #    since dp_count may be larger than the true size, 
...     #    imputed rows will be given an age of 42.0 
...     #    (also a best guess based on public information)
...     .resize(size=dp_count, constant=42.0)
...     .mean()
...     .laplace()
... )

>>> dp_mean = mean_query.release()

# /mean
