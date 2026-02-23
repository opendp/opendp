:orphan:
:nosearch:

TODO: This is the documentation we want, when polars is available.

Installation
============

The easiest way to get started with OpenDP is from Python.
Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI.

.. prompt:: bash

    pip install opendp

This will make the OpenDP modules available to your local environment.

Many of our examples begin with these lines:

.. code:: python

    .. TODO
    .. >>> import opendp.prelude as dp
    .. >>> import polars as pl
    .. >>> dp.enable_features('contrib')

The first line imports :py:mod:`opendp.prelude` under the conventional name ``dp``.

The second line imports `Polars <https://pola-rs.github.io/polars/>`_ under the conventional name ``pl``.
Polars is a dependency of OpenDP and does not need a separate ``pip install``.
OpenDP leverages Polars, but no prior familiarity with Polars is assumed in this tutorial.

The last line enables features which have not yet been vetted.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Our goal is to have all the core methods formally vetted,
but until that is complete we need to explicitly opt-in.
