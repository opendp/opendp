"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.linear_model``.

Differentially private sklearn-style linear models.
"""

from opendp._lib import import_optional_dependency

from opendp.extras.sklearn.linear_model._make_private_theil_sen import (
    make_private_theil_sen,
)

# Keep framework constructors importable without sklearn. Sphinx also supplies
# its mocked sklearn dependency here when documenting the estimators.
if import_optional_dependency("sklearn", raise_error=False) is not None:
    from opendp.extras.sklearn.linear_model._theil_sen import TheilSenRegressor
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression,
    )

__all__ = ["TheilSenRegressor", "make_private_theil_sen", "LogisticRegression"]
