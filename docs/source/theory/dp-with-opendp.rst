Differential Privacy with OpenDP
================================

This notebook brings together two threads:

-  The `previous page <a-framework-to-understand-dp.ipynb>`__ introduced
   basic DP ideas like *sensitivity* and *epsilon*, as well as terms
   that have particular meaning in OpenDP: *transformations*,
   *measures*, *measurements*, and *stability*.
-  The `User
   Guide <../api/user-guide/programming-framework/index.rst>`__
   introduced OpenDP as a programming framework, without diving into the
   mathematics.

Here we’ll see how the programming framework is related to the
underlying mathematics of differential privacy. The modular framework
helps ensure that we’re using the tools of DP appropriately, but also
has the flexibility to explore different approaches to DP.

--------------

Any functions that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            

The Laplace Mechanism
---------------------

The Laplace mechanism is a ubiquitous algorithm in the DP ecosystem that
is typically used to privatize an aggregate, like a sum or mean.

An instance of the Laplace mechanism is captured by a *measurement*
containing the following five elements:

   1. We first define the **function** :math:`f(\cdot)`, that applies
      the Laplace mechanism to some argument :math:`x`. This function
      simply samples from the Laplace distribution centered at
      :math:`x`, with a fixed noise scale.

   .. math:: f(x) = Laplace(\mu=x, b=scale)

   2. Importantly, :math:`f(\cdot)` is only well-defined for any finite
      float input. This set of permitted inputs is described by the
      **input domain** ``atom_domain(T=f64)``.

   3. The Laplace mechanism has a privacy guarantee in terms of epsilon.
      This guarantee is represented by a **privacy map**, a function
      that computes the privacy loss :math:`\epsilon` for any choice of
      sensitivity :math:`\Delta`.

   .. math:: map(\Delta) = \Delta / scale \le \epsilon

   4. This map only promises that the privacy loss will be at most
      :math:`\epsilon` if inputs from any two neighboring datasets may
      differ by no more than some quantity :math:`\Delta` under the
      absolute distance **input metric** ``absolute_distance(T=f64)``.

   5. We similarly describe units on the output (:math:`\epsilon`) via
      the **output measure** ``max_divergence()``.


The OpenDP Library consists of *constructor functions* that can be
called with simple arguments and always return valid measurements. The
``make_laplace`` constructor function returns the equivalent of the
Laplace measurement described above. Since it returns a measurement, you
can find it under ``dp.m``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # call the constructor to produce the measurement `base_lap`
            >>> base_lap = dp.m.make_laplace(
            ...     dp.atom_domain(T=float, nan=False), 
            ...     dp.absolute_distance(T=float), 
            ...     scale=5.
            ... )
            

The supporting elements on this transformation match those described
above:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> print("input domain:  ", base_lap.input_domain)
            input domain:   AtomDomain(T=f64)
            >>> print("input metric:  ", base_lap.input_metric)
            input metric:   AbsoluteDistance(f64)
            >>> print("output measure:", base_lap.output_measure)
            output measure: MaxDivergence

We now invoke the measurement on some aggregate ``0.``, to sample
:math:`Laplace(\mu=0., scale=5.)` noise:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> aggregate = 0.
            >>> print("noisy aggregate:", base_lap(aggregate))
            noisy aggregate: ...

If we are using ``base_lap`` on its own, we must know the sensitivity of
``aggregate`` (i.e. how much the aggregate can differ on two adjacent
datasets) to determine epsilon. In this case, we know ``base_lap`` has
an absolute distance input metric, so the sensitivity should represent
the greatest possible absolute distance between aggregates on adjacent
datasets.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> absolute_distance = 10.
            >>> print("epsilon:", base_lap.map(d_in=absolute_distance))
            epsilon: 2.0

This tells us that when the sensitivity is ``10``, and the noise scale
is ``5``, the epsilon consumption of a release is ``2``.

Transformation Example: Sum
---------------------------

We package computations with bounded stability into *transformations*.

A transformation that computes the sum of a vector dataset contains a
very similar set of six elements:

   1. We first define the **function** :math:`f(\cdot)`, that computes
      the sum of some argument :math:`x`.

   .. math:: f(x) = \sum x_i

   2. :math:`f(\cdot)` is only well-defined for any vector input of a
      specific type. Each element must be bounded between some lower
      bound ``L`` and upper bound ``U``. Thus the **input domain** is of
      type ``vector_domain(atom_domain(T=f64))`` with elements
      restricted between ``L`` and ``U``.

   3. The **output domain** consists of any single finite ``f64``
      scalar: ``atom_domain(T=f64)``.

   4. The sum transformation has a stability guarantee in terms of
      sensitivity. This guarantee is represented by a **stability map**,
      which is a function that computes the stability :math:`d_{out}`
      for any choice of dataset distance :math:`d_{in}`. In this case
      :math:`d_{out}` is in terms of the sensitivity.

   .. math:: map(d_{in}) = d_{in} \cdot \max(|L|, U) \le d_{out}

   5. This map only promises a sensitivity of :math:`d_{out}` under the
      assumption that neighboring datasets differ by no more than some
      quantity :math:`d_{in}` under the symmetric distance **input
      metric** ``symmetric_distance()``.

   6. The sensitivity is computed with respect to the absolute distance.
      This gives units to the output (:math:`d_{out}`) via the **output
      metric** ``absolute_distance(T=f64)``.

``make_sum`` constructs the equivalent of the sum transformation
described above. It is important to note that since the bounds are
float, the resulting transformation is calibrated to work for
floating-point numbers. You will need to be careful and intentional
about the types you use. Since it returns a transformation, you can find
it under ``dp.t``:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # call the constructor to produce the transformation `bounded_sum`
            >>> # notice that `make_sum` expects an input domain consisting of bounded data:
            >>> input_domain = dp.vector_domain(dp.atom_domain(bounds=(0., 5.)))
            >>> bounded_sum = dp.t.make_sum(input_domain, dp.symmetric_distance())
            

According to the documentation, this transformation expects a vector of
data with non-null elements bounded between ``0.`` and ``5.``. We now
invoke the transformation on some mock dataset that satisfies this
constraint. Remember that since this component is a transformation, and
not a measurement, the resulting output is not differentially private.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # under the condition that the input data is a member of the input domain...
            >>> bounded_mock_dataset = [1.3, 3.8, 0., 5.]
            >>> # ...the exact sum is:
            >>> bounded_sum(bounded_mock_dataset)
            10.1

It can help to understand a simple example of how a stability map works,
but going forward you don’t need to understand why the maps give the
numbers they give in order to use the library.

The stability argument for this transformation’s advertised sensitivity
goes roughly as follows:

   | If the input data consists of numbers bounded between 0. and 5.,
   | then the addition or removal of any one row can influence the sum
     by :math:`max(|0.|, 5.)`.
   | In addition, if one individual may contribute up to k rows,
   | then the sensitivity should further be multiplied by k.

In practice, the calculated sensitivity may be larger under certain
conditions to account for the inexactness of arithmetic on finite data
types.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # under the condition that one individual may contribute up to 2 records to `bounded_mock_dataset`...
            >>> max_contributions = 2
            >>> # ...then the sensitivity, expressed in terms of the absolute distance, is:
            >>> bounded_sum.map(d_in=max_contributions)
            10.0...

As we would expect, the sensitivity is roughly ``2 * max(|0.|, 5.)``.

Transformation Example: Clamp
-----------------------------

The sum transformation has an input domain of vectors with bounded
elements. We now construct a transformation that clamps/clips each
element to a given set of bounds.

Instead of listing the components of a clamp transformation as I’ve done
above, going forward you can check the ``**Supporting Elements**``
section of the relevant API documentation entry:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> help(dp.t.make_clamp)
            Help on function make_clamp in module opendp.transformations:
            ...

Documentation for specific types may be found behind the following
links:

-  `metrics <https://docs.rs/opendp/latest/opendp/metrics/index.html>`__
-  `measures <https://docs.rs/opendp/latest/opendp/measures/index.html>`__
-  `domains <https://docs.rs/opendp/latest/opendp/domains/index.html>`__

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> input_domain = dp.vector_domain(dp.atom_domain(T=float, nan=False))
            >>> input_metric = dp.symmetric_distance()
            
            >>> # call the constructor to produce the transformation `clamp`
            >>> clamp = dp.t.make_clamp(input_domain, input_metric, bounds=(0., 5.))
            
            >>> # `clamp` expects vectors of non-null, unbounded elements
            >>> mock_dataset = [1.3, 7.8, -2.5, 7.0]
            >>> # `clamp` emits data that is suitable for `bounded_sum`
            >>> clamp(mock_dataset)
            [1.3, 5.0, 0.0, 5.0]

According to the API documentation, the input and output metric is set
by the user. We passed in a symmetric distance metric. Therefore, the
stability map accepts a dataset distance describing the maximum number
of contributions an individual may make, and emits the same.

The stability argument for the clamp transformation is very simple:

   | If an individual may influence at most k records in a dataset, then
     after clamping each element,
   | an individual may still influence at most k records in a dataset.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # dataset distance in... dataset distance out
            >>> clamp.map(max_contributions)
            2

Chaining
--------

The OpenDP library supports chaining a transformation with a
transformation to produce a compound transformation, or a transformation
with a measurement to produce a compound measurement.

When any two compatible computations are chained, all six components of
each primitive are used to construct the new primitive.

A measurement produced from chaining a transformation with a measurement
contains the same set of six elements as in previous examples:

   1. A **function** :math:`f(\cdot)`. When you chain, the output domain
      of the transformation must match the input domain of the
      measurement.

   .. math:: f(x) = measurement(transformation(x))

   2. The **input domain** from the transformation.

   3. The **output domain** from the measurement.

   4. A **privacy_map** :math:`map(\cdot)`. When you chain, the output
      metric of the transformation must match the input metric of the
      measurement.

   .. math:: map(d_{in}) = measurement.map(transformation.map(d_{in}))

   5. The **input metric** from the transformation.

   6. The **output measure** from the measurement.

A similar logic is used when chaining a transformation with a
transformation.

We know that the

-  output domain of ``bounded_sum`` matches the input domain of
   ``base_lap``, and the
-  output metric of ``bounded_sum`` matches the input metric of
   ``base_lap``.

The same holds for ``clamp`` and ``bounded_sum``. Therefore, we can
chain all of these primitives to form a new compound measurement:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> dp_sum = clamp >> bounded_sum >> base_lap
            
            >>> # compute the DP sum of a dataset of bounded elements
            >>> print("DP sum:", dp_sum(mock_dataset))
            DP sum: ...
            
            >>> # evaluate the privacy loss of the dp_sum, when an individual can contribute at most 2 records
            >>> print("epsilon:", dp_sum.map(d_in=max_contributions))
            epsilon: ...

Retrospective
-------------

Now that you have a more thorough understanding of what’s going on, we
can breeze through an entire release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> # establish public info
            >>> max_contributions = 2
            >>> bounds = (0., 5.)
            
            >>> # construct the measurement
            >>> dp_sum = (
            ...     dp.t.make_clamp(dp.vector_domain(dp.atom_domain(T=float, nan=False)), dp.symmetric_distance(), bounds) >> 
            ...     dp.t.make_sum(dp.vector_domain(dp.atom_domain(bounds=bounds, nan=False)), dp.symmetric_distance()) >> 
            ...     dp.m.make_laplace(dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float), 5.)
            ... )
            
            >>> # evaluate the privacy expenditure and make a DP release
            >>> mock_dataset = [0.7, -0.3, 1., -1.]
            >>> print("epsilon:", dp_sum.map(max_contributions))
            epsilon: ...
            >>> print("DP sum release:", dp_sum(mock_dataset))
            DP sum release: ...

Partial Constructors
--------------------

You may notice some redundancy in the code for ``dp_sum`` above: The
output domain of a transformation will always match the input of its
successor. We can make this shorter by using ``then_*`` constructors:
These are paired with ``make_*`` constructors, but delay application of
the ``input_domain`` and ``input_metric`` arguments. We can rewrite
``dp_sum`` in an equivalent but more concise form:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            >>> dp_sum = (
            ...     (input_domain, input_metric) >>
            ...     dp.t.then_clamp((0., 5.)) >>
            ...     dp.t.then_sum() >>
            ...     dp.m.then_laplace(5.)
            ... )
            

You’ll notice that the start of the chain is special: We provide a tuple
to specify the ``input_domain`` and ``input_metric`` for ``then_clamp``.
