Multi-Table Joins
=================

OpenDP can analyze a collection of related tables, so long as the
privacy unit is described in terms of the identifier spaces that appear
across those tables.

This is useful when records at different semantic levels are stored in
different tables. For example:

* an ``events`` table may contain one row per event
* a ``users`` table may contain one row per user
* a ``households`` table may contain one row per household

The key idea is to tell OpenDP where each identifier space appears.


Identifier Sites
----------------

Use :py:class:`~opendp.mod.IdSite` to describe where an identifier space
appears in a table.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            import opendp.prelude as dp
            import polars as pl
            dp.enable_features("contrib")

            dp.IdSite(exprs="user_id", label="user")


The ``exprs`` argument accepts:

* a column name, like ``"user_id"``
* a Polars expression, like ``pl.col("user_id")``
* a list of column names or expressions

In most cases, you can think of one ``IdSite`` as describing one
appearance of one identifier space in one table.


Choosing the Protected Identifier
---------------------------------

Multi-table contexts may mention many identifier spaces, but the privacy
guarantee is always stated with respect to one of them. This is the
``protected_label``.

In the following example, the protected privacy unit is a *user*,
although the query also mentions households:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            households = pl.LazyFrame(
                {
                    "household_id": [10, 20],
                    "region": ["east", "west"],
                },
                schema={"household_id": pl.Int32, "region": pl.String},
            )
            users = pl.LazyFrame(
                {
                    "user_id": [1, 2, 3],
                    "household_id": [10, 10, 20],
                    "age": [30, 40, 50],
                },
                schema={
                    "user_id": pl.Int32,
                    "household_id": pl.Int32,
                    "age": pl.Int32,
                },
            )
            events = pl.LazyFrame(
                {
                    "user_id": [1, 1, 2, 3],
                    "value": [10, 11, 12, 13],
                },
                schema={"user_id": pl.Int32, "value": pl.Int32},
            )
            data = {
                "households": households,
                "users": users,
                "events": events,
            }

            context = dp.Context.compositor(
                data=data,
                privacy_unit=dp.unit_of(
                    contributions=1,
                    identifier={
                        "events": dp.IdSite(exprs="user_id", label="user"),
                        "users": [
                            dp.IdSite(exprs="user_id", label="user"),
                            dp.IdSite(exprs="household_id", label="household"),
                        ],
                        "households": dp.IdSite(
                            exprs="household_id", label="household"
                        ),
                    },
                    protected_label="user",
                ),
                privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
                split_evenly_over=1,
            )


Here:

* ``user`` is the privacy unit
* ``household`` is additional metadata that helps OpenDP reason about
  later joins
* the same identifier space may appear in more than one table

If your database contains more than one identifier space, it is best to
set ``protected_label`` explicitly.


Querying Across Tables
----------------------

Once the context is created, use ``context.query()`` to get a database
query object, and index into it by table name.

The following query joins ``events`` to ``users`` and then to
``households`` before truncating per protected user:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            query = (
                context.query()["events"]
                .join(
                    context.query()["users"],
                    left_on="user_id",
                    right_on="user_id",
                    how="left",
                )
                .join(
                    context.query()["households"],
                    left_on="household_id",
                    right_on="household_id",
                    how="left",
                )
                .truncate_per_group(1)
                .select(dp.len())
            )
            query.summarize()


In this example, ``query.summarize()`` reports a scale of ``1.0`` for
the private frame length query.


Joins are supported before truncation. After truncation, the query has
already been converted into an event-level stability argument, so joins
are no longer supported.


How Join Restrictions Work
--------------------------

OpenDP distinguishes two important cases:

1. A join between a private branch and a branch that does not carry the
   protected identifier. This is allowed before truncation.
2. A join between two private branches that *both* carry the protected
   identifier. This is only allowed when the join is on the active
   protected identifier itself.

For example, suppose both ``users`` and ``households`` are private with
respect to the protected label ``household``. Then joining them on
``household_id`` is allowed:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            household_context = dp.Context.compositor(
                data=data,
                privacy_unit=dp.unit_of(
                    contributions=1,
                    identifier={
                        "users": [
                            dp.IdSite(exprs="user_id", label="user"),
                            dp.IdSite(exprs="household_id", label="household"),
                        ],
                        "households": dp.IdSite(
                            exprs="household_id", label="household"
                        ),
                    },
                    protected_label="household",
                ),
                privacy_loss=dp.loss_of(epsilon=1.0),
                split_evenly_over=1,
            )
            household_query = (
                household_context.query()["users"]
                .join(
                    household_context.query()["households"],
                    left_on="household_id",
                    right_on="household_id",
                    how="left",
                )
                .truncate_per_group(1)
                .select(dp.len(scale=1.0))
            )
            household_query.release().collect()


However, if two private branches both carry the protected label
``household`` and they are joined on ``user_id`` instead, the query is
rejected because the join key is not the active protected identifier.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            # This is rejected because the protected label is "household",
            # but the join key is "user_id".
            rejected_query = (
                household_context.query()["users"]
                .join(
                    household_context.query()["users"],
                    left_on="user_id",
                    right_on="user_id",
                    how="left",
                )
                .truncate_per_group(1)
            )


Things to Remember
------------------

When building a multi-table context, keep the following mental model in
mind:

* Use one ``IdSite`` per appearance of an identifier space in a table.
* Use ``protected_label`` to choose the identifier space your privacy
  guarantee is about.
* Join before truncation.
* If two private branches both carry the protected identifier, the join
  must be on that protected identifier.
* Domain inference from ``dict[str, LazyFrame]`` infers schema only. It
  does not infer keys, uniqueness, or contribution bounds.


Advanced Note
-------------

``IdSite(exprs=[...], label=...)`` also supports multiple expressions in
one site. This is mainly for situations where the same identifier column
appears more than once, such as duplicate-identical columns after a
join. If two different columns represent two different appearances of an
identifier space, prefer two separate ``IdSite`` objects.


See Also
--------

See :doc:`identifier-truncation` for the single-table truncation story,
and the Polars API documentation for
:py:class:`~opendp.mod.IdSite` and
:py:func:`~opendp.extras.polars.LazyFrameQuery.truncate_per_group`.
