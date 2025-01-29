Serialization
=============

Most OpenDP objects can be serialized for persistence, or to share objects between a client and server.

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


:py:class:`LazyFrameQuery <opendp.extras.polars.LazyFrameQuery>` is a special case.
Although we do not support serialization of the whole object,
the plan can be extracted and then used to create a new object.

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import polars as pl

        >>> context = dp.Context.compositor(
        ...     data=pl.LazyFrame({}),
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ... ) # TODO: What's the benefit of creating this whole context, if the only thing that is serialized is the Expr?
        >>> query = context.query().select(dp.len())
        >>> plan = query._lf_plan  # TODO: Rename to ".opendp_plan"?
        >>> serialized_plan = plan.serialize()
        
        The serialized plan could be sent from a client to a server,
        where it is then deserialized:

        >>> import io
        >>> new_context = dp.Context.compositor(
        ...     data=pl.LazyFrame({}),  # The server would specify real, sensitive data here.
        ...     privacy_unit=dp.unit_of(contributions=1),
        ...     privacy_loss=dp.loss_of(epsilon=1.0),
        ...     split_evenly_over=1,
        ... ) # Do we want to be able to provide serialized_plan when creating the context? Maybe a new kwarg?
        >>> new_plan = pl.LazyFrame.deserialize(io.BytesIO(serialized_plan))      
        >>> new_query = new_context.query()
        >>> new_query._lf_plan = new_plan
        >>> new_query.release().collect().item()
        0


Note that serialized embedded Polars objects will include the path of the local binary.
These paths can be overridden at load time with the ``OPENDP_POLARS_LIB_PATH``
:ref:`environment variable <envvars>` .


Other objects, including those which are created via the plugin API,
and those which have an internal state not reflected in their constructor,
are not currently serializable:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> dp_obj = dp.user_domain("trivial_user_domain", lambda _: True)
        >>> dp.serialize(dp_obj)
        Traceback (most recent call last):
        ...
        Exception: OpenDP JSON Encoder does not handle <function <lambda> at ...>
