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

      >>> from typing import List
      >>> import opendp.prelude as dp
      >>> dp.enable_features("contrib")
      >>> context = dp.Context.compositor(
      ...     data=[1.0, 2.0, 3.0, 4.0, 5.0],
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )


      >>> sum_query = context.query().clamp((1.0, 10.0)).sum()
      
      >>> dp_sum_query = sum_query.laplace(100.0)
      >>> dp_sum_query.release()
      ...