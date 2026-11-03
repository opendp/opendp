"""
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.scipy``.

Domains for SciPy sparse arrays.
"""

from opendp.extras.scipy._domains import CSRArray2Domain, csr_array2_domain

__all__ = ["CSRArray2Domain", "csr_array2_domain"]
