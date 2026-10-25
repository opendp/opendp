
Transformation example
======================

Use :func:`~opendp.transformations.make_user_transformation` to construct your own transformation.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. tab-set::

        .. tab-item:: Python
            :sync: python

            .. literalinclude:: code/transformation.rst
                :language: python
                :dedent:
                :start-after: # enable-features
                :end-before: # /enable-features

        .. tab-item:: R
            :sync: r

            .. literalinclude:: code/transformation.R
                :language: r
                :start-after: library(opendp)
                :end-before: # make-repeat

In this example, we mock the typical API of the OpenDP library to make a transformation that duplicates each record ``multiplicity`` times:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/transformation.rst
            :language: python
            :dedent:
            :start-after: # make-repeat
            :end-before: # /make-repeat

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/transformation.R
            :language: r
            :start-after: # make-repeat
            :end-before: # /make-repeat
    
The resulting Transformation may be used interchangeably with those constructed via the library:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/transformation.rst
            :language: python
            :dedent:
            :start-after: # use-transformation
            :end-before: # /use-transformation

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/transformation.R
            :language: r
            :start-after: # use-transformation
            :end-before: # /use-transformation

The code snip may form a basis for you to create your own data transformations, 
and mix them into an OpenDP analysis.
