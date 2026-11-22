"""Experimental sklearn linear-model estimators.

The temporary `linear_model2` namespace avoids conflicts with the legacy
`linear_model.LinearRegression` wrapper. It can be merged into `linear_model`
when the legacy API has been migrated.
"""

from opendp.extras.sklearn.linear_model2._linear_regression import LinearRegression

__all__ = ["LinearRegression"]
