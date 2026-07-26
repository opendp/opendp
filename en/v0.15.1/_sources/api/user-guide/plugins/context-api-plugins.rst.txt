Context API Plugins
===================

Constructor functions built from plugins can be registered in the Context API.
This follows up on the :ref:`measurement-plugin` example,
where we built a measurement that always returns a constant value.

Constructors in the OpenDP Library almost always accept the input domain and metric as the first two arguments,
and we recommend it when building your own plugins.
When the first two arguments of the constructor function are the ``input_domain`` and ``input_metric``,
then they can be omitted when you call the function from the Context API. 
The Context API will fill them in from the compositor's input space or from the output space of the previous transformation.

.. literalinclude:: code/context-api-plugins.rst
    :language: python
    :dedent:
    :start-after: # enable-features
    :end-before: # /enable-features

.. literalinclude:: code/context-api-plugins.rst
    :language: python
    :dedent:
    :start-after: # register-anything-constant
    :end-before: # /register-anything-constant

This plugin constructor doesn't care what the input domain and input metric are,
and will happily build a measurement that always conforms with the previous transformation.
In practice, the constructor should contain checks to ensure that the input domain and input metric are meaningful for your function.

While we recommend writing constructors in this convention, 
you can still register functions that don't follow this convention.

.. literalinclude:: code/context-api-plugins.rst
    :language: python
    :dedent:
    :start-after: # register-int-constant
    :end-before: # /register-int-constant

A drawback of this approach is that the constructor function is not very flexible.
The input domain and metric are hard-coded, only accepting integers, 
and can't take into account the output domain and output metric of the previous transformation.
