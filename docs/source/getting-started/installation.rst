Installation
============

The easiest way to get started with OpenDP is from Python.
Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI.

.. prompt:: bash

    pip install opendp

This will make the OpenDP modules available to your local environment.

Almost all of our examples begin with these lines:

.. doctest::

    >>> import opendp.prelude as dp
    >>> dp.enable_features('contrib')

The first line imports :py:mod:`opendp.prelude` under the conventional name `dp`.

The second line enables features which have not yet been vetted.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Our goal is to have all the core methods formally vetted,
but until that is complete we need to explicitly opt-in.
