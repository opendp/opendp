"""This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn.linear_model2``.

Experimental scikit-learn-compatible linear regression.
The temporary ``linear_model2`` namespace avoids the legacy
``linear_model.LinearRegression`` wrapper.
"""

from opendp._lib import import_optional_dependency

# Keep low-level constructors importable without the optional sklearn install.
if import_optional_dependency("sklearn", raise_error=False) is not None:
    from opendp.extras.sklearn.linear_model2._linear_regression import LinearRegression

__all__ = ["LinearRegression"]
