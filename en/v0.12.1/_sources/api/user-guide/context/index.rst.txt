.. _context-user-guide:

Context
=======

(See also :py:mod:`opendp.context` in the API reference.)

The Context API wraps the lower-level Framework API elements.
It is currently only available for Python.
Rather than just applying noise at the end of the process,
the Context API requires you to specify your privacy budget at the start.
Because it knows the total privacy budget,
it can raise an exception and prevent you from running more queries than your budget allows.


.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> import opendp.prelude as dp
      >>> context = dp.Context.compositor(
      ...     data=[5.0] * 100,
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> sum_query = context.query().clamp((1.0, 10.0)).sum()
      >>> dp_sum_query = sum_query.laplace()
      >>> print('DP sum should be near 500:', dp_sum_query.release())  # doctest: +ELLIPSIS
      DP sum should be near 500: ...
