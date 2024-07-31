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

This demonstrates a number of low-level OpenDP patterns:

* First, define your "metric space": a data domain and a definition of distance
* Then, chain operators together to construct a ``Measurement`` (aka mechanism).
* Invoke that measurement on a value to get a DP release.

OpenDP has two APIs and we'll demonstrate how to use both:

* The **Context API** is simpler and helps to enforce best practices. Currently available only for Python.
* The **Framework API** is lower-level. Available for Python, R and Rust, it directly implements the :ref:`OpenDP Programming Framework <programming-framework>`.

Because the Context API is a wrapper around the Framework API, it is easier to use but less flexible:
All calls ultimately pass through the Framework API.

The next page will demonstrate usage of the Context API in Python, and Framework API in Python and R.
After that, the remaining "Getting Started" documentation will focus just on Python.
