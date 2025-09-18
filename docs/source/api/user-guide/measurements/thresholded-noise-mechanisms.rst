Thresholded Noise Mechanisms
============================

Thresholded noise mechanisms are used to privately release a hashmap,
where the keys are unknown and values are numbers, for example, a
histogram of counts or a set of frequencies.

The mechanism only releases key-value pairs that are “stable”. When the
value is large enough in magnitude to represent contributions from many
different individuals, then the key is not specific to any one
individual and can be released privately. The intuition is that the key
is present among all neighboring datasets.

We’ll look at the various thresholded noise mechanisms in OpenDP:

- Distribution: Laplace vs. Gaussian
- Support: float vs. integer
- Threshold: positive vs. negative
- Bit-depth

--------------

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            

Distribution: Laplace vs. Gaussian
----------------------------------

The Laplace mechanism is used to privatize an aggregate, like a sum or
mean.

An instance of the Laplace threshold mechanism is captured by a
*measurement* containing the following five elements:

.. raw:: html

   <details>

.. raw:: html

   <summary>

Elements of a Laplace Threshold Measurement

.. raw:: html

   </summary>

1. We first define the **function** :math:`f(\cdot)`, that applies the
   Laplace mechanism to the values of :math:`x`, and then discards pairs
   whose value is below the threshold.

.. code:: python

       def f(x: dict[Any, float]) -> dict[Any, float]:
           x = {k: Laplace(mu=v, b=scale) for k, v in x.items()}
           return {k: v for k, v in x.items() if v >= threshold}

2. Importantly, :math:`f(\cdot)` is only well-defined for any dictionary
   with finite float values. This set of permitted inputs is described
   by the **input domain** (denoted
   ``MapDomain<AtomDomain<TK>, AtomDomain<f64>>``).

3. The Laplace threshold mechanism has a privacy guarantee in terms of
   :math:`\epsilon` and :math:`\delta`. This guarantee is represented by
   a **privacy map**, a function that computes the greatest privacy loss
   :math:`(\epsilon, \delta)` for any choice of sensitivity
   :math:`\Delta_0, \Delta_1, \Delta_\infty`. The privacy map is roughly
   implemented as follows:

.. code:: python

       def map(d_in):
           l0, l1, li = d_in
           epsilon = l1 / scale

           # probability of sampling a noise value greater than: threshold - li
           delta_single = tail(scale, threshold - li)
           delta = 1 - (1 - delta_single)**l0
           return epsilon, delta

4. This map only promises that the privacy loss will be at most
   :math:`\epsilon` if inputs from any two neighboring datasets may
   differ by no more than some quantity
   :math:`\Delta_0, \Delta_1, \Delta_\infty` under the absolute distance
   **input metric** (``L01InfDistance<AbsoluteDistance<f64>>``).

5. We similarly describe units on the output
   (:math:`(\epsilon, \delta)`) via the **output measure**
   (``Approximate<MaxDivergence>``).

.. raw:: html

   </details>

The ``make_laplace_threshold`` constructor function returns the
equivalent of the Laplace threshold measurement described above.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> m_lap = dp.m.make_laplace_threshold(
            ...     dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=float, nan=False)),
            ...     dp.l01inf_distance(dp.absolute_distance(T=float)),
            ...     scale=1.,
            ...     threshold=20.0
            ... )
            
            >>> # invoke the measurement on some aggregate hashmap, to sample Laplace(x, 1.) noise
            >>> aggregated = {
            ...     "a": 0.0,
            ...     "b": 20.0,
            ...     "c": 40.0,
            ... }
            >>> print("noisy aggregate:", m_lap(aggregated))
            noisy aggregate: {'c': 40.17307713885866}

As expected, pairs with small values (like ``"a": 0.0``) had too few
people contribute to be included in the release.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # we must know the sensitivity of `aggregated` to determine privacy params
            >>> #  3 kinds: Δ_0, Δ_1, Δ_∞
            >>> sensitivity = 1, 1.0, 1.0
            >>> lap_eps_del = m_lap.map(d_in=sensitivity)
            >>> print("(ε, δ):", lap_eps_del)
            (ε, δ): (1.0, 2.801398224505647e-09)

``d_in`` carries three different kinds of sensitivity.

- :math:`\Delta_0`: how many values an individual may influence
- :math:`\Delta_1`: the total influence an individual may have over all
  values
- :math:`\Delta_\infty`: the influence an individual may have on any one
  value

The analogous constructor for gaussian noise is
``make_gaussian_threshold``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> m_gauss = dp.m.make_gaussian_threshold(
            ...     dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=float, nan=False)),
            ...     # NOTE: L1 is changed to L2 in the input metric
            ...     dp.l02inf_distance(dp.absolute_distance(T=float)),
            ...     scale=1.,
            ...     threshold=20.0
            ... )
            
            >>> # invoke the measurement on some aggregate hashmap, to sample Gaussian(x, 1.) noise
            >>> print("noisy aggregate:", m_gauss(aggregated))
            
            >>> # we must know the sensitivity of `aggregated` to determine privacy params
            >>> #  3 kinds: Δ_0, Δ_1, Δ_∞
            >>> sensitivity = 1, 1.0, 1.0
            >>> print("(ρ, δ):", m_gauss.map(d_in=sensitivity))
            noisy aggregate: {'c': 40.93198267967212}
            (ρ, δ): (0.5, 1.1102230246251565e-16)

In this case, :math:`\Delta_1` in ``d_in`` is replaced with
:math:`\Delta_2`.

- :math:`\Delta_0`: how many values an individual may influence
- :math:`\Delta_2`: the euclidean influence an individual may have over
  all values
- :math:`\Delta_\infty`: the influence an individual may have on any one
  value

``m_lap`` measures privacy with :math:`\epsilon` and :math:`\delta` (in
the ``Approximate<MaxDivergence>`` measure), and ``m_gauss`` measures
privacy with :math:`\rho` and :math:`\delta` (in the
``Approximate<ZeroConcentratedDivergence>`` measure).

Notice how much smaller :math:`\delta` is this time (``2.8e-9`` vs
``1.1e-16``). This is because the laplace distribution is a “fat-tailed”
distribution, meaning more of the mass of the distribution is in the
tails. The tails of the gaussian distribution decay much more quickly,
resulting in a much smaller :math:`\delta`.

For comparison, let’s convert the privacy guarantee from approx-zCDP to
compare with the thresholded laplace mechanism:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # convert ρ to an ε(δ_2) privacy profile, where total privacy loss is (ε(δ_2), δ_1 + δ_2)
            >>> m_gauss_profile = dp.c.make_zCDP_to_approxDP(m_gauss)
            >>> # fix overall δ to that used by the laplace threshold, for comparison
            >>> m_gauss_approx = dp.c.make_fix_delta(m_gauss_profile, delta=lap_eps_del[1])
            
            >>> print("(ε, δ):", m_gauss_approx.map(sensitivity))
            (ε, δ): (6.3035767282855915, 2.801398224505647e-09)

In this setting, at the same level of :math:`\delta` as the thresholded
laplace mechanism, the privacy loss of the thresholded gaussian
mechanism is over four times larger. On the other hand, the thresholded
gaussian mechanism will perform much better than the thresholded laplace
mechanism when :math:`\Delta_\infty` is small and :math:`\Delta_0` is
large. This arises when an individual has a small influence over a large
number of partitions.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> sensitivity_spread = 100, 10.0, 0.001
            >>> print("laplace  (ε, δ):", m_lap.map(d_in=sensitivity_spread))
            >>> print("gaussian (ε, δ):", m_gauss_approx.map(d_in=sensitivity_spread))
            laplace  (ε, δ): (0.1, 1.0316078580263621e-07)
            gaussian (ε, δ): (0.049969691134438526, 2.801398224505647e-09)

In this alternative world where individuals may have a small influence
on many partitions, the thresholded gaussian mechanism dominates in
utility over the thresholded laplace mechanism.

Notice that there is some redundancy in the sensitivity. Above, when an
individual may only influence 100 partitions by at most 0.001, then a
user’s total influence (:math:`\Delta_1`) could only be 0.1! Instead of
using 10, OpenDP infers :math:`\Delta_1` is
:math:`100 \cdot 0.001 = 0.1`, and :math:`\Delta_2` is
:math:`\sqrt{100} \cdot 0.001 = .01`.

Support: Float vs. Integer
--------------------------

There are also discrete analogues of the continuous Laplace and Gaussian
threshold measurements. The continuous measurements accept and emit
floats, while the discrete measurements accept and emit integers.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # call the constructor to produce the measurement `m_dlap`
            >>> input_space = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), dp.absolute_distance(T=int)
            >>> m_dlap = dp.m.make_laplace_threshold(
            ...     dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), 
            ...     dp.l01inf_distance(dp.absolute_distance(T=int)), 
            ...     scale=1.0,
            ...     threshold=10,
            ... )
            
            >>> # invoke the measurement on some integer aggregate hashmap, to sample DiscreteLaplace(x, 1.) noise
            >>> aggregated = {
            ...     "a": 0,
            ...     "b": 10,
            ...     "c": 20,
            ... }
            >>> print("noisy aggregate:", m_dlap(aggregated))
            
            >>> # in this case, the sensitivity is integral:
            >>> sensitivity = 1, 1, 1
            >>> print("(ε, δ):", m_dlap.map(d_in=sensitivity))
            noisy aggregate: {'b': 10, 'c': 22}
            (ε, δ): (1.0, 3.319000812207484e-05)

``make_gaussian_threshold`` on a discrete support is the analogous
measurement for Gaussian noise:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # call the constructor to produce the measurement `m_dgauss`
            >>> input_space = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), dp.absolute_distance(T=int)
            >>> m_dgauss = dp.m.make_gaussian_threshold(
            ...     dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), 
            ...     dp.l02inf_distance(dp.absolute_distance(T=float)), 
            ...     scale=1.0,
            ...     threshold=10,
            ... )
            
            >>> # invoke the measurement on some integer aggregate hashmap, to sample DiscreteGaussian(x, 1.) noise
            >>> aggregated = {
            ...     "a": 0,
            ...     "b": 10,
            ...     "c": 20,
            ... }
            >>> print("noisy aggregate:", m_dgauss(aggregated))
            
            >>> # in this case, the sensitivity is integral:
            >>> sensitivity = 1, 1, 1
            >>> print("(ρ, δ):", m_dgauss.map(d_in=sensitivity))
            noisy aggregate: {'c': 20, 'b': 10}
            (ρ, δ): (0.5, 1.1102230246251565e-16)

The continuous mechanisms use these discrete samplers internally.

Threshold: Positive vs. Negative
--------------------------------

When the threshold is negative, pairs with noisy values greater than the
threshold are discarded.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # call the constructor to produce the measurement `m_dlap`
            >>> input_space = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), dp.absolute_distance(T=int)
            >>> m_dlap = dp.m.make_laplace_threshold(
            ...     dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int)), 
            ...     dp.l01inf_distance(dp.absolute_distance(T=int)), 
            ...     scale=1.0,
            ...     threshold=-10,
            ... )
            
            >>> # invoke the measurement on some integer aggregate hashmap, to sample DiscreteLaplace(x, 1.) noise
            >>> aggregated = {
            ...     "a": 0,
            ...     "b": -10,
            ...     "c": -20,
            ... }
            >>> print("noisy aggregate:", m_dlap(aggregated))
            
            >>> # in this case, the sensitivity is integral:
            >>> sensitivity = 1, 1, 1
            >>> print("(ε, δ):", m_dlap.map(d_in=sensitivity))
            noisy aggregate: {'c': -20, 'b': -11}
            (ε, δ): (1.0, 3.319000812207484e-05)

Bit depth
---------

By default, all floating-point data types default to 64-bit
double-precision (denoted ``"f64"``), and all integral data types
default to 32-bit (denoted ``"i32"``). The atomic data type expected by
the function and privacy units can be further configured to operate over
specific bit-depths by explicitly specifying ``"f32"`` instead of
``"float"``, or ``"i64"`` instead of ``"int"``.

More information on acceptable data types can be found in the `Typing
section of the User Guide <../utilities/typing.rst>`__.

Desideratum: Floating-Point Granularity
---------------------------------------

Quoting from `Additive Noise Mechanisms <additive-noise-mechanisms.html#Desideratum:-Floating-Point-Granularity>`__:

    The “continuous” Laplace and Gaussian measurements convert their float
    values to a rational representation, and then add integer noise to the
    numerator via the respective discrete distribution. In the OpenDP
    Library’s default configuration, this rational representation of a float
    is exact. Therefore the privacy analysis is as tight as if you were to
    sample truly continuous noise and then postprocess by rounding to the
    nearest float.

    For most use-cases the sampling algorithm is sufficiently fast when the
    rational representation is exact. That is, when noise is sampled with a
    granularity of :math:`2^{-1074}`, the same granularity as the distance
    between subnormal 64-bit floats. However, the granularity can be
    adjusted to :math:`2^k`, for some choice of k, for a faster runtime.
    Adjusting this parameter comes with a small penalty to the sensitivity
    (to account for rounding to the nearest rational), and subsequently, to
    the privacy parameters.


In the case of additive noise mechanisms, the sensitivity from
rounding increases as a function of the vector length.

In contrast, in the case of thresholded noise mechanisms, the sensitivity from
rounding increases as a function of :math:`\Delta_0`, as only
:math:`\Delta_0` different values can round in different directions.
