Multi-Table Joins
=================

OpenDP can analyze a collection of related tables, so long as the
privacy unit is described in terms of the identifiers that appear
across those tables.

This is useful when records at different semantic levels are stored in
different tables. For example:

* an ``events`` table may contain one row per event
* a ``users`` table may contain one row per user
* a ``households`` table may contain one row per household

The key idea is to tell OpenDP where identifiers appear.
In a multi-table ``privacy_unit``, each table name maps to a
``list[bind(...)]``.

Assume data has the form:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            import opendp.prelude as dp
            import polars as pl
            dp.enable_features("contrib")

            events = pl.LazyFrame({
                "user_id": [1, 1, 2, 3],
                "value": [10, 11, 12, 13],
            })
            users = pl.LazyFrame({
                "user_id": [1, 2, 3],
                "household_id": [10, 10, 20],
                "age": [30, 40, 50],
            })
            households = pl.LazyFrame({
                "household_id": [10, 20],
                "region": ["east", "west"],
            })

            data = {
                "events": events,
                "users": users,
                "households": households,
            }


Bind to Identifier Spaces
-------------------------

Use :py:func:`~opendp.mod.bind` to describe where identifiers appear in a table.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            dp.bind("user_id", to="user")

This binding denotes that the ``"user_id"`` column contains identifiers 
in the ``"user"`` identifier space.

We now write out the bindings in the data:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            bindings = {
                "events": [
                    dp.bind("user_id", to="user")
                ],
                "users": [
                    dp.bind("user_id", to="user"),
                    dp.bind("household_id", to="household"),
                ],
                "households": [
                    dp.bind("household_id", to="household")
                ],
            }


Choose the Protected Identifier Space
-------------------------------------

Multi-table contexts may mention many identifier spaces, but the privacy
guarantee is always stated with respect to one of them. This is the
``protect`` argument.

In the following example, the protected privacy unit is a *user*:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            context = dp.Context.compositor(
                data=data,
                privacy_unit=dp.unit_of(
                    contributions=1,
                    bindings=bindings,
                    protect="user",
                ),
                privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-7),
            )


Here:

* ``user`` is the privacy unit: datasets differ by the addition or removal of one user identifier across all tables
* ``household`` is additional metadata that helps OpenDP reason about later joins
* the same identifier space may appear in more than one table


Querying Across Tables
----------------------

Once the context is created, start from ``db = context.query()`` and
build the query from ``db.table(...)``.

The following query joins ``events`` to ``users`` and then to
``households`` before truncating per protected user:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            db = context.query()
            query = (
                db.table("events")
                .join(
                    db.table("users"),
                    left_on="user_id",
                    right_on="user_id",
                    how="left",
                )
                .join(
                    db.table("households"),
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
respect to the protected space ``household``. Then joining them on
``household_id`` is allowed:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            household_context = dp.Context.compositor(
                data=data,
                privacy_unit=dp.unit_of(
                    contributions=1,
                    bindings={
                        "users": [
                            dp.bind("user_id", to="user"),
                            dp.bind("household_id", to="household"),
                        ],
                        "households": [dp.bind("household_id", to="household")],
                    },
                    protect="household",
                ),
                privacy_loss=dp.loss_of(epsilon=1.0),
                split_evenly_over=1,
            )

            household_db = household_context.query()
            household_query = (
                household_db.table("users")
                .join(
                    household_db.table("households"),
                    left_on="household_id",
                    right_on="household_id",
                    how="left",
                )
                .truncate_per_group(1)
                .select(dp.len(scale=1.0))
            )
            household_query.release().collect()


However, if two private branches both carry the protected space
``household`` and they are joined on ``user_id`` instead, the query is
rejected because the join key is not the active protected identifier.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python

            # This is rejected because the protected space is "household",
            # but the join key is "user_id".
            household_db = household_context.query()
            rejected_query = (
                household_db.table("users")
                .join(
                    household_db.table("households"),
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

* Use one ``bind`` per appearance of an identifier space in a table.
* Use ``protect`` to choose the identifier space your privacy
  guarantee is about.
* Join before truncation.
* If two private branches both carry the protected identifier, the join
  must be on that protected identifier.
* Domain inference from ``dict[str, LazyFrame]`` infers schema only. It
  does not infer keys, uniqueness, or contribution bounds.


Advanced Note
-------------

The ``exprs`` argument in ``bind`` accepts:

* a column name, like ``"user_id"``
* a Polars expression, like ``pl.col("user_id")``
* a list of column names or expressions

In most cases, you can think of one ``bind`` as describing one
appearance of one identifier space in one table.

A list ``bind(exprs=[...], to=...)`` is useful when the same identifier column
appears more than once, such as duplicate-identical columns after a
join. If two different columns represent two different appearances of an
identifier space, prefer two separate ``bind`` objects.


See Also
--------

See :doc:`identifier-truncation` for the single-table truncation story,
and the Polars API documentation for
:py:func:`~opendp.mod.bind` and
:py:func:`~opendp.extras.polars.LazyFrameQuery.truncate_per_group`.
