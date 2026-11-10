"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn``.
Submodule organization will follow the conventions of `scikit-learn <https://scikit-learn.org/stable/api/index.html>`_.
"""

from typing import TYPE_CHECKING

from opendp.extras.sklearn._make_sklearn_fit import (
    make_sklearn_fit,
    make_dataframe_to_sklearn_fit,
)

from opendp.extras.sklearn._domains import (
    SklearnFit,
    SklearnFitDescriptor,
    sklearn_fit_domain,
)

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.sklearn import decomposition, linear_model


def __getattr__(name):
    # Load estimator modules only when used, so other OpenDP APIs don't require sklearn.
    if name not in __all__:
        raise AttributeError(name)
    from importlib import import_module
    from opendp._lib import import_optional_dependency

    import_optional_dependency("sklearn")
    if name == "FrozenTransformer":
        member = getattr(import_module(f"{__name__}._frozen"), name)
        globals()[name] = member
        return member
    if name == "ColumnTransformer":
        member = import_module(f"{__name__}.compose").ColumnTransformer
        globals()[name] = member
        return member
    if name == "Pipeline":
        member = import_module(f"{__name__}.pipeline").Pipeline
        globals()[name] = member
        return member
    module = import_module(f"{__name__}.{name}")
    return module


__all__ = [
    "FrozenTransformer",
    "SklearnFit",
    "SklearnFitDescriptor",
    "make_sklearn_fit",
    "make_dataframe_to_sklearn_fit",
    "sklearn_fit_domain",
    "decomposition",
    "linear_model",
    "compose",
    "ColumnTransformer",
    "Pipeline",
]
