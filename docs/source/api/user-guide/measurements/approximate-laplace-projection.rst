.. _approximate-laplace-projection:

Approximate Laplace Projection
==============================

When dealing with data that has an unknown key-set, the keys themselves
need to be protected. One approach (shown in this section) is to release
a differentially private low-dimensional projection of the key-space.
The mechanism releases a queryable that can be queried with keys to
retrieve noisy count estimates.

Two other commonly-used approaches to deal with unknown key-set are
“explicit” key release, where only pre-specified keys are released, and
“stable” key release, where only keys contributed by many individuals
are released. (Stable key release uses thresholded noise mechanisms.)

--------------

Any constructors that have not completed the proof-writing and vetting
process may still be accessed if you opt-in to “contrib”. Please contact
us if you are interested in proof-writing. Thank you!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> import opendp.prelude as dp
            >>> dp.enable_features("contrib")
            

The mechanism takes as input a hash map, typically where the keys
represent grouping keys, and values represent the number of occurences
of those keys in the dataset.

In this example, imagine you have a dataset where the keys correspond to
the first names of individuals, and values correspond to the number of
people with that name in a dataset.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> # Based on real-world name frequencies. 
            >>> # The data could just as well consist of many hundreds of more names.
            >>> names = {'Michael': 750, 'James': 500, 'Sharon': 50}
            >>> input_domain = dp.map_domain(dp.atom_domain(T=str), dp.atom_domain(T=int))
            

Just like the thresholded noise mechanism, the sensitivity is expressed
in terms of a triple:

- :math:`\Delta_0`: how many values an individual may influence
- :math:`\Delta_1`: the total influence an individual may have over all
  values
- :math:`\Delta_\infty`: the influence an individual may have on any one
  value

Naturally, in this setting, all three of these quantities are one:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> sensitivity = 1, 1, 1
            >>> input_metric = dp.l01inf_distance(dp.absolute_distance(T=int))
            

A key part of the mechanism is determining how large the projected space
should be. The size of the projected space is associated with a
memory/utility tradeoff, where a larger projected space costs more
memory, but retains greater representational capacity. If the projected
space is too small, then many more strings will hash to the same value,
resulting in a higher likelihood of hash collisions. The size of the
projected space should ideally be large enough that it is unlikely more
than one frequent name hashes to the same value.

The two most important factors that influence the size of the projected
space are:

- ``total_limit``: upper-bound the sum of values.
- ``value_limit``: upper-bound each value. Required if the input domain
  is not already bounded.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> epsilon = 1.0
            
            >>> m_alp = dp.binary_search_chain(
            ...     lambda s: dp.m.make_alp_queryable(
            ...         input_domain, input_metric, scale=s, total_limit=50000, value_limit=1000
            ...     ),
            ...     d_in=sensitivity,
            ...     d_out=epsilon,
            ... )
            
            >>> qbl = m_alp(names)
            

This mechanism releases a queryable containing a differentially private,
hash-based representation of the counts of all possible names.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: pycon

            >>> qbl("Michael"), qbl("James"), qbl("Sharon"), qbl("Lancelot")
            (...)

These counts roughly correspond to the input data.

Notice that these counts work out to multiples of four. To reduce the
size of the projection, the precision of answers in the compressed
representation is controlled via a parameter ``alpha``, which has a
default of four.

Finally, the size of the projected space can be scaled via the
``size_multiplier`` argument, which is set to a default of fifty.
``alpha`` and ``size_multiplier``, together with ``total_limit`` and
``value_limit``, comprise a heuristic to determine a reasonable domain
size.
