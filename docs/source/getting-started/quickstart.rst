Quickstart
==========

OpenDP is available for Python and R.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI:

        .. prompt:: bash

            pip install opendp

    .. tab-item:: R
        :sync: r

        Launch R and then use ``install.packages``:

        .. code:: r

            install.packages('opendp', repos = 'https://opendp.r-universe.dev/')


This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: quickstart/context.rst
            :language: python
            :start-after: init
            :end-before: /init

    .. tab-item:: R
        :sync: r

        .. literalinclude:: quickstart/framework.R
            :language: r
            :start-after: init
            :end-before: /init

Once you've installed OpenDP, you can write your first program.
Let's apply Laplace noise to a value.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: quickstart/context.rst
            :language: python
            :start-after: demo
            :end-before: /demo

    .. tab-item:: R
        :sync: r

        .. literalinclude:: quickstart/framework.R
            :language: r
            :start-after: demo
            :end-before: /demo

This is obviously not the easiest way to add noise to a number,
but it demonstrates a number of OpenDP patterns:

* Defining your metric space with ``space_of`` in Python.
* Chaining operators together with ``>>`` in Python, or ``|>`` in R.
* Constructing a ``Measurement`` function on your metric space with ``then_base_laplace``.
* Invoking that measurement on a value to get a DP release.
