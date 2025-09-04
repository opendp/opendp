.. _development-environment:

Development Environment
=======================

If you are writing code, the first task to tackle is setting up the development environment.

You will need to check out the code, and at a minimum, build the Rust binaries.

Most developers will also install Python and/or R:
If you are only interested in developing a feature in one of these languages,
you will not need to set up the other.

.. contents:: |toctitle|
    :local:

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


Rust Setup
----------

If you have not already, install the `Rust toolchain <https://www.rust-lang.org/tools/install>`_.

Make sure you are on the latest Rust version:

.. code-block:: bash

    rustup update

Now run ``cargo build`` in the ``rust`` subdirectory of the repo:

.. code-block:: bash

    cd rust
    cargo build --all-features

This will compile a debug build of the OpenDP shared library, placing it in the directory ``opendp/rust/target/debug``. 
(The specific name of the library file will vary depending on your platform.)

Substitute ``cargo build`` with ``cargo test`` to test, or ``cargo check`` to check syntax.

.. note::

  If linking fails when runnning ``cargo test``, you may need to build without the ``extension-module`` feature (`context here <https://pyo3.rs/v0.25.1/faq.html#i-cant-run-cargo-test-or-i-cant-build-in-a-cargo-workspace-im-having-linker-issues-like-symbol-not-found-or-undefined-reference-to-_pyexc_systemerror>`_):

  .. code-block:: bash

    cargo test --no-default-features --features=partials,use-openssl,untrusted,polars,ffi,derive
  
  Since ``extension-module`` is a default feature, 
  the other remaining features are enabled in the above command.


Note that Python and R require builds with different features.
Details are in the :ref:`python-setup` and :ref:`r-setup` sections below.
Setting a feature changes how the crate compiles.

.. _rust-feature-listing:

.. dropdown:: Comprehensive Rust Feature List

    
   .. list-table::
      :widths: 25 75
      :header-rows: 1

      * - Name
        - Description
      * - ``polars``
        - Enable for functionality related to the Polars dataframe library.
      * - ``contrib``
        - Enable to include constructors that have not passed the vetting process.
      * - ``honest-but-curious``
        - Enable to include constructors whose differential privacy (or stability) properties
          rely on the constructor arguments being correct.
          That is, if a user/adversary is 'honest' in specifying the constructor arguments,
          then even if they later become 'curious' and try to learn something from the measurement outputs,
          they will not be able to violate the differential privacy promises of the measurement.
      * - ``floating-point``
        - Enable to include transformations and measurements with floating-point vulnerabilities.
      * - ``untrusted``
        - Enables untrusted features ``contrib``, ``honest-but-curious`` and ``floating-point``.
      * - ``ffi``
        - Enable to include C foreign function interfaces.
      * - ``derive``
        - Enable to support code generation and links to proofs in documentation.
      * - ``extension-module``
        - Enable creation of a python extension module for use by Python Polars. Must be disabled for Rust tests. Enable for Python builds to skip linking to ``libpython``.
      * - ``bindings``
        - Enable to generate Python and R source code. Also enables the ``ffi``, ``derive`` and ``extension-module`` features. 
      * - ``partials``
        - Enabled by default. When enabled, ``then_*`` functions are generated from ``make_*`` functions. Also enables the ``derive`` feature.
      * - ``use-openssl``
        - Enabled by default. Use OpenSSL for secure noise generation.

   A list of features available in bindings languages (R, Python) can be found in :ref:`feature-listing`.

To make the crate compile faster, FFI functions in debug builds support a reduced set of primitive types.
Release-mode builds support the full set of primitive types and undergo compiler optimizations, but take longer to compile.
You can compile a release build by adding the ``--release`` flag.
In contrast to debug builds, release builds are located in ``opendp/rust/target/release``.
To use a release-mode binary from the Python bindings, 
set the environment variable ``OPENDP_TEST_RELEASE=1`` before importing OpenDP.

For more on our Rust programming patterns:

.. toctree::

    rust-initiation

.. _python-setup:

Python Setup
------------

First, build a debug binary that works with Python. (Note that the resulting binary will not work with R.)

.. code-block:: bash

    cd rust
    cargo build --all-features

If you only need to regenerate the Python bindings, this is sufficient:

.. code-block:: bash

    cargo check --all-features

If you have not already, install `Python version 3.10 or higher <https://www.python.org>`_.

You can install a local Python package that uses your new OpenDP binary. 

.. dropdown:: Optional Virtual Environment

    We recommend setting up a virtual environment first, but this is optional:

    .. code-block:: bash

        # recommended. conda is just as valid
        cd python
        python3 -m venv .venv-dev
        source .venv-dev/bin/activate


Change to the ``python`` directory, install dependencies, and then install the Python OpenDP library itself.

.. code-block:: bash

    cd python

    pip install -r requirements-dev.txt
    pip install -e '.[mbi]'

``requirement-dev.txt`` is compiled from ``requirements-dev.in``:
To update dependencies, follow the directions in that file.

In the second line, the ``-e`` flag is significant! 
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
    

Python Tests
^^^^^^^^^^^^

You can test that things are working by running OpenDP's Python test suite, using ``pytest``.
Run the tests from the ``python`` directory. 

.. code-block:: bash

    pytest -v

If everything has gone well, you'll see a bunch of output, then a line similar to this:

.. prompt:: bash

    ================== 57 passed in 1.02s ==================

If pytest is not found, don't forget to activate your virtual environment!

This is just a quick overview of building OpenDP. 

Python Documentation
^^^^^^^^^^^^^^^^^^^^

This documentation website is built with Sphinx.
The source code and developer documentation is
`here <https://github.com/opendp/opendp/tree/main/docs#readme>`_.


.. _r-setup:

R Setup
-------

First, build a debug binary that works with R. (Note that the resulting binary will not work with Python.)

.. code-block:: bash

    cd rust
    cargo build --features untrusted,bindings

If you have not already, `install R <https://cran.r-project.org/>`_.

Then, set an environment variable to the absolute path of the OpenDP Library binary directory:

.. code-block:: bash

    export OPENDP_LIB_DIR=`realpath target/debug`

The default R install for MacOS also includes GUI elements like Tcl/Tk,
so for the smoothest development experience we suggest these additional installs:

.. code-block:: bash

    brew install harfbuzz fribidi libgit2 xquartz

Then, install devtools in R:

.. code-block:: R

    install.packages(c("devtools", "RcppTOML", "lintr"))

After each edit to the R or Rust source, run the following command in R to (re)load the R package:

.. code-block:: R

    devtools::load_all("R/opendp/", recompile=TRUE)

.. This function...
.. - runs `src/Makevars`
..     - cargo builds `libopendp.a` (rust-lib) and `opendp.h` (rust-lib header file)
.. - compiles the c files in `src/`, which statically links with `libopendp.a`
.. - outputs `src/opendp.so`, which is used by all R functions
.. - reloads all R functions

To do a full package installation from local sources:

.. prompt:: bash

    tools/r_stage.sh && Rscript -e 'devtools::install("R/opendp")'

To restore to a developer setup, run:

.. prompt:: bash

    tools/r_stage.sh -c



R Tests
^^^^^^^

Run tests (tests are located in ``R/opendp/tests/``):

.. code-block:: R

    devtools::test("R/opendp")


R also has a built-in check function that runs tests and checks for common errors:

.. code-block:: R
    
    devtools::check("R/opendp")

To run the same check manually, use:

.. code-block:: bash

    R CMD build R/opendp
    R CMD check opendp_*.tar.gz --as-cran

It is important ``R CMD check`` is run on the ``.tar.gz``, not on ``R/opendp``, 
because ``check`` depends on some of the changes ``build`` makes within the ``.tar.gz``.


R Documentation
^^^^^^^^^^^^^^^

This script uses roxygen to generate ``R/opendp/man`` pages from ``#'`` code comments,
and then uses ``pkgdown`` to render the documentation website.

.. code-block:: bash

    tools/r_stage.sh -d


Docs Setup
----------

The documentation build is described in the `docs/README.md <https://github.com/opendp/opendp/tree/main/docs#readme>`_.

For more on proof writing patterns:

.. toctree::

    proof-initiation

Release Process
-----------------

Our `release process <https://github.com/opendp/opendp/tree/main/.github/workflows#making-a-release>`_
uses github workflows.

.. _envvars:

Environment Variables
---------------------

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Name
     - Description
   * - ``OPENDP_LIB_DIR``
     - Overrides the directory in which the OpenDP language binding looks for the OpenDP Library binary.  
       See example in :ref:`r-setup`. 
   * - ``OPENDP_POLARS_LIB_PATH``
     - Each OpenDP Polars plugin contains a path to the OpenDP Library binary.
       When OpenDP is used as a query server, library paths in queries submitted by clients are stale (local to the client).
       This environment variable overrides paths in new OpenDP Polars plugins and OnceFrames.
       For Python, you can read this value from ``opendp._lib.lib_path`` (read-only).
       This is separate from ``OPENDP_LIB_DIR`` because we anticipate it diverging for R.
   * - ``OPENDP_HEADLESS``
     - Used by CI. When ``true``, The Python ``opendp`` package will import without the presence of the OpenDP Library binary.
   * - ``OPENDP_SPHINX_PORT`` and ``OPENDP_SPHINX_URI``
     - When configured, links to proof documents hosted by Sphinx point to the URI and port.
       The URI defaults to localhost. 
       Allows for a local documentation site.
       Start the server from ``docs/`` with ``make sphinx-server``.
   * - ``OPENDP_RUSTDOC_PORT`` and ``OPENDP_RUSTDOC_URI``
     - When configured, links in proof documents to Rustdocs include the URI and port. 
       The URI defaults to localhost. 
       Allows for a local documentation site.
       Start the server from ``docs/`` with ``make rustdoc-server``. 
   * - ``OPENDP_TEST_RELEASE``
     - When ``true``, and ``OPENDP_LIB_DIR`` is set, 
       the library will attempt to load the ``release`` binary instead of the ``debug`` binary.

Developer Tooling
-----------------

There are many development environments that work with Rust and LaTex. Here are a few:

* `VS Code <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_
* `Intellij IDEA <https://plugins.jetbrains.com/plugin/8182-rust>`_
* `Sublime <https://github.com/rust-lang/rust-enhanced>`_

Use whatever tooling you are comfortable with.


A few notes on VS Code:

* Be sure to install the `rust-analyzer <https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer>`_ plugin, not the Rust plugin
* Open ``rust-analyzer``'s extension settings, search "features" and add ``"untrusted", "bindings"``
* Look for ``Problems`` in the bottom panel for live compilation errors as you work
* Other useful extensions are "Better Toml", "crates" and "LaTex Workshop"
* To configure VS Code with suggested tasks and settings: ``cp -a .vscode-suggested .vscode``


A few notes on Intellij IDEA:

* Both Intellij IDEA community edition and the CodeWithMe plugin are free
* Be sure to open the project at the root of the git repository
* Be sure to install the Python and Rust plugins for interactivity
* Be sure to "attach" the Cargo.toml in the red banner the first time you open a Rust source file
* Use run configurations to `build the Rust library <https://plugins.jetbrains.com/plugin/8182-rust/docs/cargo-command-configuration.html#cargo-command-config>`_ and run tests
