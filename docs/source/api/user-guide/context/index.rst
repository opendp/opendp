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
      >>> dp.enable_features("contrib")
      >>> context = dp.Context.compositor(
      ...     data=[1, 2, 3],
      ...     privacy_unit=dp.unit_of(contributions=3),
      ...     privacy_loss=dp.loss_of(epsilon=3.0),
      ...     split_evenly_over=1,
      ...     domain=dp.domain_of(List[int]),
      ... )