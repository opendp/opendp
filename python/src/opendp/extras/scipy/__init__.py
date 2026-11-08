"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.scipy``.

Domains for SciPy sparse arrays.
"""

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from opendp.extras.scipy._domains import CSRArray2Descriptor, csr_array2_domain

__all__ = ["CSRArray2Descriptor", "csr_array2_domain"]


def __getattr__(name):
    if name not in __all__:
        raise AttributeError(name)
    from importlib import import_module
    from opendp._lib import import_optional_dependency

    import_optional_dependency("scipy")
    return getattr(import_module(f"{__name__}._domains"), name)
