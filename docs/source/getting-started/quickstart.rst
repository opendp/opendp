Quickstart
==========

OpenDP is available for Python and R.

.. tabs::

    .. group-tab:: Python

        Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI:

        .. prompt:: bash

            pip install opendp

    .. group-tab:: R

        Launch R and then use ``install.packages``:

        .. code:: r

            install.packages('opendp', repos = 'https://opendp.r-universe.dev/')

This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> from opendp.mod import enable_features
            >>> enable_features('contrib')

    .. group-tab:: R

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: 1-library
            :end-before: 2-use

Once you've installed OpenDP, you can write your first program.
Let's apply Laplace noise to a value.

.. tabs::

    .. group-tab:: Python

        .. doctest::

            >>> import opendp.prelude as dp
            >>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
            >>> dp_value = base_laplace(123.0)

    .. group-tab:: R

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: 2-use

This is obviously not the easiest way to add noise to a number,
but it demonstrates a number of OpenDP patterns:

* Defining your metric space with ``space_of`` in Python.
* Chaining operators together with ``>>`` in Python, or ``|>`` in R.
* Constructing a ``Measurement`` function on your metric space with ``then_base_laplace``.
* Invoking that measurement on a value to get a DP release.
