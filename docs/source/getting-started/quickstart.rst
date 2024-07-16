Quickstart
==========

OpenDP is available for Python, R, and Rust.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        Use ``pip`` to install the `opendp <https://pypi.org/project/opendp/>`_ package from PyPI:

        .. prompt:: bash

            pip install opendp

    .. tab-item:: R
        :sync: r

        First, make sure `Rust is installed <https://www.rust-lang.org/tools/install>`_.
        (The R package includes Rust source code which will be compiled during install.)

        Then, launch R and use ``install.packages``:

        .. code:: r

            install.packages('opendp', repos = 'https://opendp.r-universe.dev/')

    .. tab-item:: Rust
        :sync: rust

        In a new directory run ``cargo init`` and then specify OpenDP as a dependency in ``Cargo.toml``:

        .. literalinclude:: code/quickstart-framework-rust/Cargo-for-docs.toml
            :language: toml
            :start-after: init
            :end-before: /init


This will make the OpenDP modules available to your local environment.

The vetting process is currently underway for the code in the OpenDP Library.
Any code that has not completed the vetting process is marked as "contrib" and will not run unless you opt-in.
Enable ``contrib`` globally with the following snippet:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/quickstart-context.rst
            :language: python
            :start-after: init
            :end-before: /init

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/quickstart-framework.R
            :language: r
            :start-after: init
            :end-before: /init

    .. tab-item:: Rust
        :sync: rust

        In Rust, ``contrib`` is specified in ``Cargo.toml``.


Once you've installed OpenDP, you can write your first program.
Let's apply Laplace noise to a value.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/quickstart-context.rst
            :language: python
            :start-after: demo
            :end-before: /demo

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/quickstart-framework.R
            :language: r
            :start-after: demo
            :end-before: /demo

    .. tab-item:: Rust
        :sync: rust

        .. literalinclude:: code/quickstart-framework-rust/src/main.rs
            :language: rust
            :start-after: demo
            :end-before: /demo

This is obviously not the easiest way to add noise to a number,
but it demonstrates a number of low-level OpenDP patterns:

* Defining your metric space with ``space_of`` in Python's Context API, or a (domain, distance) tuple in any language..
* Chaining operators together with ``>>`` in Python and Rust, or ``|>`` in R.
* Constructing a ``Measurement`` function on your metric space with ``then_laplace``.
* Invoking that measurement on a value to get a DP release.

OpenDP has layered APIs which provide increasing abstraction and usability:

* The **Framework API** is low-level. Available for Python and R, it mirrors the underlying Rust framework.
* The **Context API** introduces a ``Context`` class which ensures that queries do not exceed the privacy budget. Currently available only for Python.
* The **Polars API** provides a DP extension to the `Polars <https://docs.pola.rs/>`_ dataframe library. Currently available only for Python.

Because the higher-level APIs are built on the Framework API, they are easier to use but less flexible: All calls ultimately pass through the Framework API.

This page and the next will use the Framework and Context APIs to demonstrate the similarities between the Framework APIs in different languages.
The remaining documentation focuses on the Polars API with Python.
