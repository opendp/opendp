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

    .. tab-item:: Rust
        :sync: rust

        Reference the ``opendp`` `crate <https://crates.io/crates/opendp>`_ in your ``Cargo.toml`` file:

        .. code:: toml

            [dependencies]
            opendp = { version = "0.9.2", features = ["contrib"] }


This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. doctest::

            >>> from opendp.mod import enable_features
            >>> enable_features('contrib')

    .. tab-item:: R
        :sync: r

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: 1-library
            :end-before: 2-use

    .. tab-item:: Rust
        :sync: rust

        In Rust, ``contrib`` features are enabled in your ``Cargo.toml`` in the previous step.

Once you've installed OpenDP, you can write your first program.
Let's apply Laplace noise to a value.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. doctest::

            >>> import opendp.prelude as dp
            >>> base_laplace = dp.space_of(float) >> dp.m.then_base_laplace(scale=1.)
            >>> dp_value = base_laplace(123.0)

    .. tab-item:: R
        :sync: r

        .. literalinclude:: quickstart.R
            :language: r
            :start-after: 2-use

    .. tab-item:: Rust
        :sync: rust

        .. literalinclude:: quickstart.rs
            :language: rust
            :start-after: 2-use

This is obviously not the easiest way to add noise to a number,
but it demonstrates a number of OpenDP patterns:

* Defining your metric space with ``space_of`` in Python.
* Chaining operators together with ``>>`` in Python, or ``|>`` in R.
* Constructing a ``Measurement`` function on your metric space with ``then_base_laplace``.
* Invoking that measurement on a value to get a DP release.
