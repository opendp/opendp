Plugins
=======

Because Differential Privacy is a wide and expanding field,
we can't implement every mechanism for every user,
but users can provide their own code through these methods:

Domains
    :py:func:`user_domain <opendp.domains.user_domain>`
Measurements
    :py:func:`make_user_measurement <opendp.measurements.make_user_measurement>` and :py:func:`then_user_measurement <opendp.measurements.then_user_measurement>`
Measures
    :py:func:`user_divergence <opendp.measures.user_divergence>`
Metrics
    :py:func:`user_distance <opendp.metrics.user_distance>`
Transformations
    :py:func:`make_user_transformation <opendp.transformations.make_user_transformation>`
Postprocessors
    :py:func:`new_function <opendp.core.new_function>`
Privacy Profiles
    :py:func:`new_privacy_profile <opendp.measures.new_privacy_profile>`

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
  selecting-grouping-columns
