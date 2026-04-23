.. _measurement-plugin:

Measurement example
===================

Use :func:`~opendp.measurements.make_user_measurement` to construct a measurement for your own mechanism.

.. note::

    This requires a looser trust model, as we cannot verify any privacy or stability properties of user-defined functions.

    .. tab-set::

        .. tab-item:: Python
            :sync: python

            .. literalinclude:: code/measurement.rst
                :language: python
                :dedent:
                :start-after: # enable-features
                :end-before: # /enable-features

        .. tab-item:: R
            :sync: r

            .. literalinclude:: code/measurement.R
                :language: r
                :start-after: library(opendp)
                :end-before: # make-base-constant

This example mocks the typical API of the OpenDP library to make the *most private* DP mechanism ever!

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/measurement.rst
            :language: python
            :dedent:
            :start-after: # make-base-constant
            :end-before: # /make-base-constant

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/measurement.R
            :language: r
            :start-after: # make-base-constant
            :end-before: # /make-base-constant
    
The resulting Measurement may be used interchangeably with those constructed via the library:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/measurement.rst
            :language: python
            :dedent:
            :start-after: # use-measurement
            :end-before: # /use-measurement

    .. tab-item:: R
        :sync: r

        .. literalinclude:: code/measurement.R
            :language: r
            :start-after: # use-measurement
            :end-before: # /use-measurement

While this mechanism clearly has no utility, 
the code snip may form a basis for you to create own measurements, 
or even incorporate mechanisms from other libraries.
