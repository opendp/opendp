.. _context-user-guide:

Context
=======

(See also :py:mod:`opendp.context` in the API reference.)

The Context API wraps the lower-level Framework API elements.
Rather than just applying noise at the end of the process,
the Context API requires you to specify your privacy budget at the start.
Because it knows the total privacy budget,
it can raise an exception and prevent you from running more queries than your budget allows.


.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

      >>> import opendp.prelude as dp
      >>> context = dp.Context.compositor(
      ...     data=[5.0] * 100,
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> sum_query = (
      ...     context.query()
      ...     .impute_constant(0.0)
      ...     .clamp((1.0, 10.0))
      ...     .sum()
      ... )
      >>> dp_sum_query = sum_query.laplace()
      >>> print(
      ...     "DP sum should be near 500:", dp_sum_query.release()
      ... )  # doctest: +ELLIPSIS
      DP sum should be near 500: ...

  .. tab-item:: R

    .. code-block:: r

      library(opendp)
      enable_features("contrib", "idealized-numerics")

      context <- Context$compositor(
        data = rep(5.0, 100),
        privacy_unit = unit_of(contributions = 1L),
        privacy_loss = loss_of(epsilon = 1),
        split_evenly_over = 1L
      )

      sum_query <- query(context) |>
        then_impute_constant(0.0) |>
        then_clamp(c(1.0, 10.0)) |>
        then_sum() |>
        then_laplace(auto())

      # Parameter search inspects the query without spending privacy budget.
      param(sum_query, .T = "float")
      dp_sum <- release(sum_query, .T = "float")

      current_privacy_loss(context)
      remaining_privacy_loss(context)

In R, a query is piped through the existing ``then_*`` constructors. Passing
``auto()`` for one numeric constructor argument asks the Context API to find a
value that satisfies the query's assigned privacy loss. Building a query,
calling ``resolve()``, or inspecting ``param()`` does not consume privacy
budget; only a successful ``release()`` does.

Providing ``split_evenly_over`` or ``split_by_weights`` creates a static
sequence of query allowances. When neither is supplied, the context uses a
privacy filter and each query may declare its own loss, such as
``query(context, epsilon = 0.25)``. A context with an infinite total loss uses
an odometer: it tracks accumulated loss but has no finite remaining budget to
report.

Queries may also create nested compositors. Transformations before
``compositor()`` define the child context's query space, and their stability
map is applied to the child context's ``d_in``.
