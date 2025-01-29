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


Objects which have an internal state not reflected in their constructor
are not currently serializable:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> dp.serialize(dp.Queryable('value', 'query_type'))
        Traceback (most recent call last):
        ...
        Exception: OpenDP JSON Encoder currently does not handle instances of <class 'opendp.mod.Queryable'>: It may have state which is not set by the constructor. Error on: Queryable(Q=query_type)

Plugins with user defined functions are serializable,
but ``honest-but-curious`` must be enabled,
and due to limitations of python's pickle, ``lambda`` will not work.

.. We can't provide an example of UDFs in a doctest,
.. because pickle is unable to locate the function definition in this context.

Note that serialized embedded Polars objects will include the path of the local binary.
These paths can be overridden at load time with the ``OPENDP_POLARS_LIB_PATH``
:ref:`environment variable <envvars>` .