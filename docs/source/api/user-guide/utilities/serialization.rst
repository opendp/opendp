Serialization
=============

.. _lazyframe-serialization:

LazyFrameQuery Serialization
----------------------------

For :py:class:`LazyFrameQuery <opendp.extras.polars.LazyFrameQuery>`,
the plan can be extracted and then used to create a new object,
perhaps on a remote server.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import polars as pl

        >>> context = dp.Context.compositor( # First, on the client...
        ...     data=pl.LazyFrame({}), # we might have dummy data here.
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ... )
        >>> query = context.query().select(dp.len())
        >>> serialized_plan = query.polars_plan.serialize()

        >>> new_context = dp.Context.compositor( # Then, on the server...
        ...     data=pl.LazyFrame({}),  # real, sensitive data here.
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ... )
        >>> # Then use the serialized plan from the client:
        >>> new_query = new_context.deserialize_polars_plan(serialized_plan)
        >>> print('DP len:', new_query.release().collect().item())
        DP len: ...


Note that serialized embedded Polars objects will include the path of the local binary.
These paths can be overridden at load time with the ``OPENDP_POLARS_LIB_PATH``
:ref:`environment variable <envvars>` .


Context Serialization
---------------------

``LazyFrameQuery`` serialization is a special case of context serialization.
Instead of ``query.polars_plan`` we'll use ``query.resolve()``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

      >>> context = dp.Context.compositor(
      ...     data=[1, 2, 3],  # dummy data on the client
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> query = context.query().clamp((0, 10)).sum().laplace()
      >>> measure = query.resolve()
      >>> serialized_measure = dp.serialize(measure)

      >>> new_context = dp.Context.compositor(
      ...     data=[1, 2, 3],  # sensitive, real data on the server
      ...     privacy_unit=dp.unit_of(contributions=1),
      ...     privacy_loss=dp.loss_of(epsilon=1.0),
      ...     split_evenly_over=1,
      ... )
      >>> new_query = new_context.query()
      >>> new_query.chain = dp.deserialize(serialized_measure)


Framework API Serialization
---------------------------

At a lower level, Framework API objects can be serialized and deserialized
with ``dp.serialize()`` and ``dp.deserialize()``.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import opendp.prelude as dp
        >>> dp.enable_features('contrib')
        >>> dp_obj = ((dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
        ...     >> dp.t.then_clamp((0, 10))
        ...     >> dp.t.then_sum()
        ...     >> dp.m.then_laplace(scale=5.0)
        ... )
        >>> serialized = dp.serialize(dp_obj)
        >>> serialized[:32]
        '{"__function__": "make_chain_mt"'


While the serialization format is JSON, we do not guarantee any stability between versions,
and we discourage users from writing their own JSON.
If this is something you need, please reach out so that we can understand your use case.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> new_obj = dp.deserialize(serialized)
        >>> type(dp_obj)
        <class 'opendp.mod.Measurement'>
        >>> type(new_obj)
        <class 'opendp.mod.Measurement'>


Limitations
-----------

Objects created with the plugin API and context objects, discussed above, are not currently serializable:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> dp_obj = dp.user_domain("trivial_user_domain", lambda _: True)
        >>> dp.serialize(dp_obj)
        Traceback (most recent call last):
        ...
        Exception: OpenDP JSON Encoder does not handle <function <lambda> at ...>
