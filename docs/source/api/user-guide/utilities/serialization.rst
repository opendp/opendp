Serialization
=============

.. _lazyframe-serialization:

LazyFrameQuery Serialization
----------------------------

For a :py:class:`~opendp.extras.polars.LazyFrameQuery`,
the plan can be extracted and then used to create a new object,
perhaps on a remote server.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> import polars as pl

        >>> context = dp.Context.compositor(  # First, on the client...
        ...     data=pl.LazyFrame({}),  # we might have dummy data here.
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(dp.len(signed=True))
        >>> serialized_plan = query.polars_plan.serialize()

        >>> new_context = (
        ...     dp.Context.compositor(  # Then, on the server...
        ...         data=pl.LazyFrame({}),  # real, sensitive data here.
        ...         privacy_unit=dp.unit_of(contributions=1),
        ...         privacy_loss=dp.loss_of(epsilon=1.0),
        ...         split_evenly_over=1,
        ...     )
        ... )
        >>> # Then use the serialized plan from the client:
        >>> new_query = new_context.query().deserialize(serialized_plan)
        >>> print("DP len:", new_query.release().collect().item())
        DP len: ...


The ``polars_plan`` is a Polars ``LazyFrame``. The binary plan is produced by `Polars serialization <https://docs.pola.rs/api/python/stable/reference/lazyframe/api/polars.LazyFrame.serialize.html#polars.LazyFrame.serialize>`_.

For plans received from untrusted clients, use
:py:meth:`~opendp.extras.polars.LazyFrameQuery.deserialize` with its default
``trusted=False``, rather than calling ``polars.LazyFrame.deserialize``
directly. OpenDP checks and sanitizes the serialized DSL before Polars can
unpickle Python callbacks or load native plugins. Python callbacks and
pickled plugin arguments are rejected. Only OpenDP's supported DP plugins
are allowed, and library paths are replaced with the OpenDP library loaded
by the server.

File scans are rejected by default. To read public files on the server,
pass their paths in ``allowed_files`` when deserializing. The plan may only
reference those exact files. A server can also check
``len(serialized_plan)`` to set its own request-size limit.

For locally created, trusted plans, ``deserialize(..., trusted=True)``
skips the security checks and permits ordinary Polars features, including
local file scans and Python callbacks. Never set ``trusted=True`` for plans
from untrusted clients. The regular OpenDP FFI data conversion path also
assumes its caller is trusted.

Context Serialization
---------------------

``LazyFrameQuery`` serialization is a special case.
Where Context is used without Polars, we have ``query.resolve()`` instead of ``query.polars_plan``.

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

      >>> context = dp.Context.compositor(
      ...     data=[1, 2, 3],  # dummy data on the client
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> query = context.query().clamp((0, 10)).sum().laplace()
      >>> measurement = query.resolve()
      >>> serialized_measurement = dp.serialize(measurement)

      >>> new_context = dp.Context.compositor(
      ...     data=[1, 2, 3],  # sensitive, real data on the server
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> new_query = new_context.query()
      >>> new_query.chain = dp.deserialize(serialized_measurement)


Framework API Serialization
---------------------------

While the serialization of measurements is more likely to be useful,
any object from the Framework API can also be serialized and deserialized the same way.
(In fact, the serialization of a measurement relies on recursively serializing its components.)

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> domain = dp.vector_domain(dp.atom_domain(T=int))
        >>> serialized_domain = dp.serialize(domain)
        >>> new_domain = dp.deserialize(serialized_domain)
        >>> assert type(domain) == type(new_domain)

        >>> serialized_domain[:32]
        '{"__function__": "vector_domain"'


While the serialization format is JSON, we do not guarantee stability between versions,
and we discourage users from writing their own JSON.
If this is something you need, please reach out so that we can understand your use case.


Limitations
-----------

Objects created with the plugin API and context objects, discussed above, are not currently serializable:

.. tab-set::

  .. tab-item:: Python

    .. code:: pycon

        >>> dp_obj = dp.user_domain(
        ...     "trivial_user_domain", lambda _: True
        ... )
        >>> dp.serialize(dp_obj)
        Traceback (most recent call last):
        ...
        Exception: OpenDP JSON Encoder does not handle <function <lambda> at ...>

        >>> dp.serialize(context)
        Traceback (most recent call last):
        ...
        Exception: OpenDP JSON Encoder does not handle instances of <class 'opendp.context.Context'>...
