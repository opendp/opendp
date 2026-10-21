"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn``.
Submodule organization will follow the conventions of `scikit-learn <https://scikit-learn.org/stable/api/index.html>`_.
"""

from opendp.extras.sklearn._domains import (
    SklearnFit, SklearnFitDomain, sklearn_fit_domain,
)

def __getattr__(name):
    # Load estimator modules only when used, so other OpenDP APIs don't require sklearn.
    if name not in __all__:
        raise AttributeError(name)
    from importlib import import_module
    from opendp._lib import import_optional_dependency

    import_optional_dependency("sklearn")
    module = import_module(f"{__name__}.{name}")
    globals()[name] = module
    return module

__all__ = [
    "SklearnFit", "SklearnFitDomain", "sklearn_fit_domain",
    "cluster", "decomposition", "linear_model",
]
