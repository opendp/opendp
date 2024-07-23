'''
This module requires extra installs: ``pip install opendp[scikit-learn]``

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.sklearn``.    
'''

from .pca import PCA