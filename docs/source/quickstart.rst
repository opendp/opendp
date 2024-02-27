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

            install.packages('opendp', repos = 'https://opendp-dev.r-universe.dev')

This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. tabs::

    .. group-tab:: Python

        .. literalinclude:: quickstart.py
            :language: python
            :start-after: init
            :end-before: /init

    .. group-tab:: R

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: init
            :end-before: /init

Hello, OpenDP!
--------------

Once you've installed OpenDP, you can write your first program.
In the example below, we'll construct a Laplace mechanism of type :class:`opendp.mod.Measurement`, 
then invoke it on a scalar aggregate.

.. tabs::

    .. group-tab:: Python

        .. literalinclude:: quickstart.py
            :language: python
            :start-after: dp-demo
            :end-before: /dp-demo

    .. group-tab:: R

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: dp-demo
            :end-before: /dp-demo

This code snip uses a number of OpenDP concepts:

* Defining your metric space up-front with ``space_of`` in Python.
* Chaining operators together with ``>>`` in Python, or ``|>`` in R.
* Constructing a ``Measurement`` on your metric space with ``then_base_laplace``.
* Invoking the ``base_laplace`` measurement on a value to get a DP release.

If you would like to skip directly to a more complete example, see :ref:`putting-together`.

Otherwise, continue on to the User Guide, where these concepts are explained in detail.
