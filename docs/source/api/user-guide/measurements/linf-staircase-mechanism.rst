L-infinity Staircase Mechanism
==============================

The L-infinity staircase mechanism is an additive noise mechanism for
releasing vector-valued statistics with bounded coordinatewise sensitivity.
It is appropriate when adjacent datasets can change each coordinate by at most
``d_in`` in absolute value, and this bound does not grow with the number of
coordinates.

Equivalently, use this mechanism when the upstream transformation's output
metric is ``LInfDistance``. The mechanism samples joint cube-shaped noise,
instead of adding independent coordinatewise noise.

This page documents :func:`~opendp.measurements.make_linf_staircase`.

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


When to Use It
--------------

The L-infinity staircase mechanism is useful when the natural sensitivity of a
vector-valued statistic is a maximum coordinate change:

.. math::

   d_\infty(x, x') = \max_j |x_j - x'_j|.

This can arise for vectors of scores, bounded coordinatewise summaries, or
postprocessed vectors whose stability proof controls each coordinate
uniformly. In these settings, the privacy analysis depends on the largest
coordinate change, not on the sum of changes across coordinates.

If your stability guarantee is instead an L1 bound, use
:func:`~opendp.measurements.make_l1_staircase` or
:func:`~opendp.measurements.make_laplace`. If your privacy definition is zCDP
and your stability guarantee is an L2 bound, use
:func:`~opendp.measurements.make_gaussian`.


Mechanism
---------

For a vector-valued statistic ``x`` in dimension ``d``, the continuous
L-infinity staircase mechanism samples a scale index

.. math::

   \Pr[I = i] \propto (i \delta + r)^d \exp(-\epsilon i),

then samples ``Y`` uniformly from the L-infinity unit ball,
``[-1, 1]^d``, and releases

.. math::

   x + (I \delta + r)Y,

rounded to the output floating-point type.

When the carrier type is integral, OpenDP instead uses the lattice analogue:
it samples an integer L-infinity radius with cube-shell-count weighting, then
samples uniformly from the corresponding integer cube shell.

In both cases, the mechanism is vector-valued. Scalar inputs are supported as
one-dimensional convenience wrappers. In one dimension, L1 and L-infinity
coincide.


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
            ...     dp.linf_distance(T=float),
            ... )
            >>> meas = dp.m.make_linf_staircase(
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

The privacy map rounds the L-infinity sensitivity up to a whole number of
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
            ...     dp.linf_distance(T=int),
            ... )
            >>> meas = dp.m.make_linf_staircase(
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

            >>> scalar_meas = dp.m.make_linf_staircase(
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

For example, with ``delta=2`` and ``epsilon=0.5``, an L-infinity sensitivity
of ``3`` maps to ``ceil(3 / 2) * 0.5 = 1.0``.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> meas = dp.m.make_linf_staircase(
            ...     dp.vector_domain(dp.atom_domain(T=float, nan=False)),
            ...     dp.linf_distance(T=float),
            ...     delta=2.0,
            ...     r=1.0,
            ...     epsilon=0.5,
            ... )
            >>> meas.map(d_in=3.0)
            1.0


Comparison With L1 Staircase
----------------------------

The L1 and L-infinity staircase constructors have similar APIs, but they apply
to different stability statements:

.. list-table::
   :header-rows: 1

   * - Sensitivity statement
     - Constructor
     - Noise geometry
   * - The sum of coordinate changes is bounded.
     - :func:`~opendp.measurements.make_l1_staircase`
     - L1 ball / integer L1 sphere
   * - The maximum coordinate change is bounded.
     - :func:`~opendp.measurements.make_linf_staircase`
     - L-infinity ball / integer cube shell

Choose the constructor whose metric matches the output metric of the
transformation you are chaining from.

