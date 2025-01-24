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


Some objects, including those which are created via the plugin API,
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


Another option to consider, if you are using the Polars interface,
is to use `Polars serialization <https://docs.pola.rs/api/python/dev/reference/expressions/api/polars.Expr.meta.serialize.html#polars.Expr.meta.serialize>`_ directly:

.. tab-set::

  .. tab-item:: Python

    .. code:: python

        >>> import polars as pl
        >>> import io

        >>> expr = dp.len(scale=1.0)
        >>> bytes = expr.meta.serialize()
        >>> new_expr = pl.Expr.deserialize(bytes)
        >>> type(expr)
        <class 'polars.expr.expr.Expr'>
        >>> type(new_expr)
        <class 'polars.expr.expr.Expr'>

Note that serialized Polars objects will include the path of the local binary.
These paths can be overridden at load time with the ``OPENDP_POLARS_LIB_PATH``
:ref:`environment variable <envvars>` .