Development Environment
=======================

Follow the steps below to get an OpenDP development environment set up, including the ability to run tests in both Rust and Python.

.. contents:: Contents:
    :local:

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

Clone the repo and change into the ``opendp`` directory that's created.

.. code-block:: bash

    git clone https://github.com/opendp/opendp.git
    cd opendp

Build OpenDP
------------

Change to the ``rust`` directory before attempting a build, run the tests, and then return to the ``opendp`` directory.

.. code-block:: bash

    cd rust
    cargo build
    cargo test
    cd ..

Add `--features=untrusted` to the `cargo` commands to include non-secure floating-point and contrib features like `make_base_laplace`.
If you are on Windows, [check the windows build instructions](https://github.com/opendp/opendp/tree/main/windows).

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
