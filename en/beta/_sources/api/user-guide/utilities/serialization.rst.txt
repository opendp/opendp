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
        >>> query = context.query().select(dp.len())
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
        >>> new_query = new_context.deserialize_polars_plan(
        ...     serialized_plan
        ... )
        >>> print("DP len:", new_query.release().collect().item())
        DP len: ...


Note that the ``polars_plan`` is really just a Polars ``LazyFrame``, and ``deserialize_polars_plan`` is a convenience method for `Polars binary serialization <https://docs.pola.rs/api/python/stable/reference/lazyframe/api/polars.LazyFrame.serialize.html#polars.LazyFrame.serialize>`_.

Polars serializations will include the path of the local binary.
These paths can be overridden at load time with the ``OPENDP_POLARS_LIB_PATH`` :ref:`environment variable <envvars>`.


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
