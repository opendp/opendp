.. _development-environment:

Development Environment
=======================

Follow the steps below to get an OpenDP development environment set up, including the ability to run tests in both Rust and Python.

Install Rust
------------

Download Rust from the `Rust website`_.

.. _Rust website: https://www.rust-lang.org

Install Python
--------------

Download Python from the `Python website`_.

.. _Python website: https://www.python.org

Clone the OpenDP Repo
---------------------

If you don't have write access to the OpenDP repository, you will first need to make a fork.
`The GitHub documentation explains this process well <https://docs.github.com/en/get-started/quickstart/fork-a-repo>`_.

Clone the repo (or your fork) and change into the ``opendp`` directory that's created.

.. code-block:: bash

    git clone https://github.com/opendp/opendp.git
    cd opendp

Building OpenDP
===============

Build OpenDP
------------

Change to the ``rust`` directory before attempting a build, run the tests, and then return to the ``opendp`` directory.

.. code-block:: bash

    cd rust
    cargo build
    cargo test
    cd ..

Add `--features=untrusted` to the `cargo` commands to include non-secure floating-point and contrib features like `make_base_laplace`.
Refer to the :ref:`developer-faq` section if you run into compilation problems.

Install Python Dependencies
---------------------------

Change to the ``python`` directory, create a Python virtual environment, activate it, install dependencies, and then install the Python OpenDP library itself.

.. code-block:: bash

    cd python
    python3 -m venv venv
    source venv/bin/activate
    pip install flake8 pytest
    mkdir src/opendp/lib
    pip install -e .

The developer install will not work if you don't use the `-e` flag when installing with pip!

Run the Tests
-------------

From the ``python`` directory, set an environment variable to the location of the OpenDP library you built. Then run the tests.

.. code-block:: bash

    export OPENDP_LIB_DIR=../rust/target/debug
    pytest -v

Documentation
=============

Documentation Source
--------------------

The source for this documentation can be found in the "docs" directory at https://github.com/opendp/opendp

Building the Docs
-----------------

The docs are built using Sphinx and the steps are listed in the README in the "docs" directory.


Tooling
=======

There are many development environments that work with Rust. Here are a few:

* `Intellij IDEA <https://plugins.jetbrains.com/plugin/8182-rust>`_
* `VS Code <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust>`_
* `Sublime <https://github.com/rust-lang/rust-enhanced>`_

Use whatever developer tooling you are comfortable with.
The benefit to using Intellij IDEA is that the core developers use it,
which makes it possible for one of us to actually join your IDE with the `CodeWithMe Plugin <https://www.jetbrains.com/code-with-me/>`_,
and talk through issues.

A few notes on Intellij IDEA:

* Both the Intellij IDEA community edition and the CodeWithMe plugin are free
* Be sure to open the project at the root of the git repository
* Be sure to install the Python and Rust plugins for interactivity
* Be sure to "attach" the Cargo.toml in the red banner the first time you open a Rust source file

To reiterate, of course, use whatever developer tooling you are comfortable with!
