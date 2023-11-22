.. _development-environment:

Development Environment
=======================
If you are writing code, the first task to tackle is setting up the development environment.

Follow the steps below to set up an OpenDP Library development environment, including the ability to run tests in both Rust and Python.

* Install the `Rust toolchain <https://www.rust-lang.org/tools/install>`_.
* Install `Python version 3.8 or higher <https://www.python.org>`_.


Clone the OpenDP Repo
---------------------

If you want to submit PRs, but don't have write access to the OpenDP repository, you will either need to request to join the organization or make a fork.
`The GitHub documentation explains forking <https://docs.github.com/en/get-started/quickstart/fork-a-repo>`_.

Clone the repo (or your fork) and change into the ``opendp`` directory that's created.

.. code-block:: bash

    git clone git@github.com:opendp/opendp.git
    cd opendp


If you have not `set up SSH <https://docs.github.com/en/authentication/connecting-to-github-with-ssh>`_, you can clone with https instead:

.. code-block:: bash

    git clone https://github.com/opendp/opendp.git


Build OpenDP
------------

Next, you'll need to build the Rust binaries. 

Make sure you are on the latest Rust version:

.. code-block:: bash

    rustup update

Now run ``cargo build`` in the ``rust`` subdirectory of the repo:

.. code-block:: bash

    cd rust
    cargo build --features untrusted,bindings-python

This will compile a debug build of the OpenDP shared library, placing it in the directory ``opendp/rust/target/debug``. 
(The specific name of the library file will vary depending on your platform.)

Substitute ``cargo build`` with ``cargo test`` to test, or ``cargo check`` to run a lightweight check that the code is valid.

In the above commands, the features ``untrusted`` and ``bindings-python`` are enabled.

Setting a feature changes how the crate compiles:


.. raw:: html

   <details style="margin:-1em 0 2em 2em">
   <summary><a>Feature List</a></summary>

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Name
     - Description
   * - ``untrusted``
     - Enables untrusted features ``contrib`` and ``floating-point``.
   * - ``contrib``
     - Enable to include constructors that have not passed the vetting process.
   * - ``honest-but-curious``
     - Enable to include constructors that are only private if the constructor arguments are honest.
   * - ``floating-point``
     - Enable to include transformations/measurements with floating-point vulnerabilities.
   * - ``bindings-python``
     - Enables the ``ffi`` and ``derive`` feature and regenerates sources in the Python package.
   * - ``ffi``
     - Enable to include C foreign function interfaces.
   * - ``derive``
     - Enable to embed links to proofs in the documentation.
   * - ``use-system-libs``
     - Enable to use the system installation of MPFR.
   * - ``use-mpfr``
     - Already enabled. Use MPFR for exact floating-point arithmetic.
   * - ``use-openssl``
     - Already enabled. Use OpenSSL for secure noise generation.

.. raw:: html

   </details>


To make the crate compile faster, ffi functions in debug builds support a reduced set of primitive types.
Release-mode builds support the full set of primitive types and undergo compiler optimizations, but take longer to compile.
You can compile a release build by adding the ``--release`` flag.
In contrast to debug builds, release builds are located in ``opendp/rust/target/release``.
To use a release-mode binary from the Python bindings, 
set the environment variable ``OPENDP_TEST_RELEASE=1`` before importing OpenDP.

If you run into problems, please contact us!

.. note::

    You may encounter the following build error on Windows:

    .. code-block::

        error: failed to run custom build command for `gmp-mpfr-sys v1.4.10`

    There is a more involved `setup guide <https://github.com/opendp/opendp/tree/main/rust/windows>`_ for Windows users.
    You can compromise to simple and vulnerable builds instead, by adding the ``--no-default-features`` flag to cargo commands.
    Be advised this flag disables GMP's exact float handling, as well as OpenSSL's secure noise generation.


Python Setup
------------

You can install a local Python package that uses your new OpenDP binary. 

We recommend setting up a virtual environment first, but this is optional:

.. raw:: html

   <details style="margin:-1em 0 2em 2em">
   <summary><a>Virtual Environment</a></summary>

.. code-block:: bash

    # recommended. conda is just as valid
    cd opendp
    python3 -m venv .venv
    source .venv/bin/activate

.. raw:: html

   </details>

Change to the ``python`` directory, install dependencies, and then install the Python OpenDP library itself.

.. code-block:: bash

    cd python

    pip install flake8 pytest wheel
    pip install -e .

The `-e` flag is significant! 
It stands for "editable", meaning you only have to run this command once.
That is, you do not need to reinstall the OpenDP Python package if changes are made in the ``/python/src`` folder or to the library binary,
but you should restart the Python interpreter or kernel.

At this point, you should be able import OpenDP as a locally installed package:

.. code-block:: python

    import opendp


.. note::

    If you encounter the following error on import:
    
    .. code-block::

        OSError: dlopen ... (mach-o file, but is an incompatible architecture)
    
    You should check that the architecture from ``rustc -vV`` matches your Python architecture.
    This can occur if you are on a Mac M1 and have an x86_64 Python install.
    

Testing Python
--------------
You can test that things are working by running OpenDP's Python test suite, using ``pytest``.
Run the tests from the ``python`` directory. 

.. code-block:: bash

    pytest -v

If everything has gone well, you'll see a bunch of output, then a line similar to this:

.. prompt:: bash

    ================== 57 passed in 1.02s ==================

If pytest is not found, don't forget to activate your virtual environment!

This is just a quick overview of building OpenDP. 
If you're interested in porting OpenDP to a different platform, we'd be delighted to get your help; please :doc:`contact us <../contact>`!

Documentation
-------------

The source for this documentation website can be found in the "docs" directory at https://github.com/opendp/opendp.
The docs are built using Sphinx and the steps to build it are listed in the README in the "docs" directory.


Developer Tooling
-----------------

There are many development environments that work with Rust and LaTex. Here are a few:

* `VS Code <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_
* `Intellij IDEA <https://plugins.jetbrains.com/plugin/8182-rust>`_
* `Sublime <https://github.com/rust-lang/rust-enhanced>`_

Use whatever tooling you are comfortable with.


A few notes on VS Code:

* Be sure to install the `rust-analyzer <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_ plugin, not the Rust plugin
* Open ``rust-analyzer``'s extension settings, search "features" and add ``"untrusted", "bindings-python"``
* Look for ``Problems`` in the bottom panel for live compilation errors as you work
* Other useful extensions are "Better Toml", "crates" and "LaTex Workshop"
* To configure VS Code with suggested tasks and settings: ``cp -a .vscode-suggested .vscode``


A few notes on Intellij IDEA:

* Both Intellij IDEA community edition and the CodeWithMe plugin are free
* Be sure to open the project at the root of the git repository
* Be sure to install the Python and Rust plugins for interactivity
* Be sure to "attach" the Cargo.toml in the red banner the first time you open a Rust source file
* Use run configurations to `build the Rust library <https://plugins.jetbrains.com/plugin/8182-rust/docs/cargo-command-configuration.html#cargo-command-config>`_ and run tests
