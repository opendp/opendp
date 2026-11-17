API User Guide
==============

This user guide gives a top-down picture of OpenDP;
it complements the bottom-up view provided by the `Python API Reference <../python/index.html>`_.


.. _feature-listing:

Feature Listing
---------------

Features can be enabled via the following syntax:

.. tab-set::

   .. tab-item:: Python
      :sync: python

      .. code-block:: python

         import opendp.prelude as dp
         dp.enable_features("contrib")

   .. tab-item:: R
      :sync: r

      .. code-block:: R

         library(opendp)
         enable_features("contrib")

   .. tab-item:: Rust
      :sync: rust

      Edit the dependency on OpenDP in your Cargo.toml:

      .. code-block:: toml

         opendp = { features = ["contrib"] }
      

Features that are available from Python and R:

.. list-table::
   :widths: 25 75
   :header-rows: 1

   * - Name
     - Description
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
   * - ``rust-stack-trace``
     - Enable to allow stack traces to include stack frames from Rust.

See also the :ref:`comprehensive listing of features for Rust<rust-feature-listing>`.


.. toctree::
   :titlesonly:
   :maxdepth: 1

   limitations
   programming-framework/index
   transformations/index
   measurements/index
   combinators/index
   utilities/index
   context/index
   polars/index
   plugins/index
