Plugins
=======

Because Differential Privacy is a wide and expanding field,
we can't implement every mechanism for every user,
but users can provide their own code for each component of the OpenDP Framework through these methods:

Domains
    :py:func:`user_domain <opendp.domains.user_domain>`
Measurements
    :py:func:`make_user_measurement <opendp.measurements.make_user_measurement>` and :py:func:`then_user_measurement <opendp.measurements.then_user_measurement>`
Measures
    :py:func:`user_divergence <opendp.measures.user_divergence>`
Metrics
    :py:func:`user_distance <opendp.metrics.user_distance>`
Transformations
    :py:func:`make_user_transform <opendp.transformations.make_user_transformation>`

Examples
--------

.. toctree::
  :titlesonly:

  selecting-grouping-columns