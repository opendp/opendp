.. _combinators-user-guide:

Combinators
===========

(See also :py:mod:`opendp.combinators` in the API reference.)

Combinator constructors use Transformations or Measurements to produce a new Transformation or Measurement.

.. _chaining:

Chaining
--------

Chainers are used to incrementally piece Transformations or Measurements together that represent longer computational pipelines.

.. list-table::
   :header-rows: 1

   * - Function
     - From
     - To
   * - :func:`make_chain_tt <opendp.combinators.make_chain_tt>`
     - Transformation
     - Transformation
   * - :func:`make_chain_mt <opendp.combinators.make_chain_mt>`
     - Transformation
     - Measurement
   * - :func:`make_chain_pm <opendp.combinators.make_chain_pm>`
     - Measurement
     - Transformation

However, OpenDP overloads ``>>`` in Python, and ``|>`` in R, as shortcuts,
so you may never need to reference these long forms directly.
For more on the uses and limitations of chaining:

.. toctree::
    :titlesonly:

    chaining

Composition
-----------

OpenDP has several compositors for making multiple releases on the same dataset:

.. list-table::
   :header-rows: 1

   * - Function
     - Queries
     - Privacy Loss
   * - :func:`make_composition <opendp.combinators.make_composition>`
     - Non-interactive
     - Non-interactive
   * - :func:`make_adaptive_composition <opendp.combinators.make_adaptive_composition>`
     - Interactive
     - Non-interactive
   * - :func:`make_fully_adaptive_composition <opendp.combinators.make_fully_adaptive_composition>`
     - Interactive
     - Interactive

Composition combinators can compose Measurements with ``ZeroConcentratedDivergence``, ``MaxDivergence`` and ``FixedSmoothedMaxDivergence`` output measures,
and arbitrary input metrics and domains.

Each of these is described in more detail:

.. toctree::
    :titlesonly:

    non-adaptive-composition
    adaptive-composition
    fully-adaptive-composition
    privacy-filters
    compositor-chaining-and-nesting


Other Topics
------------

There are just a couple other applications of combinators we should mention for completeness:

.. toctree::
    :titlesonly:

    measure-casting
    amplification