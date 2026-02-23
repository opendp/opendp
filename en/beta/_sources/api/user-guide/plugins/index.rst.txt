.. _plugins:

Plugins
=======

Because Differential Privacy is a wide and expanding field,
we can't implement every mechanism for every user,
but users can provide their own code through these methods:

Measurements
    :py:func:`~opendp.measurements.make_user_measurement`
Transformations
    :py:func:`~opendp.transformations.make_user_transformation`
Domains
    :py:func:`~opendp.domains.user_domain`
Metrics
    :py:func:`~opendp.metrics.user_distance`
Measures
    :py:func:`~opendp.measures.user_divergence`
Postprocessors
    :py:func:`~opendp.core.new_function`
Privacy Profiles
    :py:func:`~opendp.measures.new_privacy_profile`

OpenDP itself uses the plugin machinery in some cases.
It is usually easier to prototype ideas in Python than in Rust,
so this provides a lower barrier to entry to contributing to OpenDP.
If the contribution proves to be useful to the broader community,
it can then be translated to Rust.


Examples
---------------

.. toctree::
  :titlesonly:

  measurement
  transformation
  context-api-plugins
  theil-sen-regression
  selecting-grouping-columns
