'''
This module requires extra installs: ``pip install 'opendp[scikit-learn]'``

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp

The members of this module will then be accessible at ``dp.sklearn``.
Submodule organization will follow the conventions of `scikit-learn <https://scikit-learn.org/stable/api/index.html>`_.
'''

import opendp.extras.sklearn.decomposition as decomposition
import opendp.extras.sklearn.linear_model as linear_model