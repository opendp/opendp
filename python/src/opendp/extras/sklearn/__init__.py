'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.sklearn``.
Submodule organization will follow the conventions of `scikit-learn <https://scikit-learn.org/stable/api/index.html>`_.
'''

import opendp.extras.sklearn.decomposition