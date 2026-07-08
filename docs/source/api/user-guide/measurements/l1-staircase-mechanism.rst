L1 Staircase Mechanism
======================

The L1 staircase mechanism is an additive noise mechanism for releasing
vector-valued statistics with bounded L1 sensitivity.
It is similar in use to :func:`~opendp.measurements.make_laplace`,
but samples noise from a joint L1 staircase distribution instead of adding
independent coordinatewise noise.

This page documents :func:`~opendp.measurements.make_l1_staircase`.

--------------

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt in to ``contrib``.
Please contact us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")


Mechanism
---------

For a vector-valued statistic ``x`` in dimension ``d``, the continuous L1
staircase mechanism samples a scale index

.. math::

   \Pr[I = i] \propto (i \delta + r)^d \exp(-\epsilon i),

then samples ``Y`` uniformly from the L1 unit ball and releases

.. math::

   x + (I \delta + r)Y,

rounded to the output floating-point type.

When the carrier type is integral, OpenDP instead uses the lattice analogue:
it samples an integer L1 radius with the appropriate shell-count weighting,
then samples uniformly from the integer L1 sphere at that radius.

In both cases, the mechanism is vector-valued. Scalar inputs are supported
as one-dimensional convenience wrappers over the same vector mechanism.


Floating-Point Inputs
---------------------

When the input domain is over floats, the mechanism uses the continuous
sampler and rounds the final release to the carrier type.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(T=float, nan=False)),
            ...     dp.l1_distance(T=float),
            ... )
            >>> meas = dp.m.make_l1_staircase(
            ...     *input_space,
            ...     delta=1.0,
            ...     r=1.0,
            ...     epsilon=1.0,
            ... )

            >>> release = meas([0.0, 2.0, 4.0])
            >>> len(release)
            3
            >>> all(isinstance(x, float) for x in release)
            True

            >>> meas.map(d_in=1.0)
            1.0

The privacy map rounds the L1 sensitivity up to a whole number of
``delta``-sized groups:

.. math::

   d_{out} = \left\lceil \frac{d_{in}}{\delta} \right\rceil \epsilon.


Integer Inputs
--------------

When the input domain is over integers, the mechanism samples lattice noise.
The public constructor is the same; the input domain determines the support.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(T=int)),
            ...     dp.l1_distance(T=int),
            ... )
            >>> meas = dp.m.make_l1_staircase(
            ...     *input_space,
            ...     delta=2.0,
            ...     r=1.0,
            ...     epsilon=1.0,
            ... )

            >>> release = meas([0, 2, 4])
            >>> len(release)
            3
            >>> all(isinstance(x, int) for x in release)
            True

            >>> meas.map(d_in=1)
            1.0

For integer inputs, ``delta`` and ``r`` are interpreted as integer staircase
parameters. In the current implementation, ``delta`` must be a positive
integer and ``r`` must be in ``1..=delta``.


Scalar Convenience Wrapper
--------------------------

The scalar form uses ``AtomDomain`` and ``AbsoluteDistance``. It is a
one-dimensional wrapper around the vector-valued mechanism.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> scalar_meas = dp.m.make_l1_staircase(
            ...     dp.atom_domain(T=float, nan=False),
            ...     dp.absolute_distance(T=float),
            ...     delta=1.0,
            ...     r=1.0,
            ...     epsilon=1.0,
            ... )

            >>> isinstance(scalar_meas(0.0), float)
            True
            >>> scalar_meas.map(d_in=1.0)
            1.0


Choosing Parameters
-------------------

The three constructor parameters control the staircase shape:

.. list-table::
   :header-rows: 1

   * - Parameter
     - Meaning
   * - ``delta``
     - Width of each privacy-loss step in the input metric.
   * - ``r``
     - Offset inside each step. For floating-point carriers, ``r`` may lie
       in ``[0, delta]``. For integer carriers, ``r`` must be integral and
       in ``1..=delta``.
   * - ``epsilon``
     - Privacy cost per full ``delta`` step.

For example, with ``delta=2`` and ``epsilon=0.5``, an L1 sensitivity of
``3`` maps to ``ceil(3 / 2) * 0.5 = 1.0``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> meas = dp.m.make_l1_staircase(
            ...     dp.vector_domain(dp.atom_domain(T=float, nan=False)),
            ...     dp.l1_distance(T=float),
            ...     delta=2.0,
            ...     r=1.0,
            ...     epsilon=0.5,
            ... )
            >>> meas.map(d_in=3.0)
            1.0


Chaining
--------

The L1 staircase mechanism chains with transformations whose output metric is
``L1Distance`` for vector releases, or ``AbsoluteDistance`` for scalar releases.
For example, the bounded sum transformation produces a scalar release, so it can
chain directly into the scalar convenience form of ``then_l1_staircase``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> input_space = (
            ...     dp.vector_domain(dp.atom_domain(bounds=(0.0, 10.0)), size=10),
            ...     dp.symmetric_distance(),
            ... )
            >>> private_sum = (
            ...     input_space
            ...     >> dp.t.then_sum()
            ...     >> dp.m.then_l1_staircase(
            ...         delta=10.0,
            ...         r=1.0,
            ...         epsilon=1.0,
            ...     )
            ... )

            >>> isinstance(private_sum([1.0] * 10), float)
            True
            >>> private_sum.map(d_in=1)
            1.0
