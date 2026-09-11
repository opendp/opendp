API Comparison
==============

OpenDP has two APIs:

* The **Context API** is simpler and helps enforce best practices. Currently available only for Python, it is used in the :doc:`../../../getting-started/index` documentation.
* The **Framework API** is lower-level. Available for Python, R and Rust, it directly implements the :doc:`OpenDP Programming Framework <../../../theory/a-framework-to-understand-dp>`.

Because the Context API is a wrapper around the Framework API, it is easier to use but less flexible:
All calls ultimately pass through the Framework API.

A side-by-side comparison may make the differences more clear.

1. Identify the Unit of Privacy
-------------------------------

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: unit-of-privacy
            :end-before: /unit-of-privacy

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: unit-of-privacy
            :end-before: /unit-of-privacy

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: unit-of-privacy
            :end-before: /unit-of-privacy

2. Set Privacy Loss Parameters
------------------------------

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: privacy-loss
            :end-before: /privacy-loss

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: privacy-loss
            :end-before: /privacy-loss

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: privacy-loss
            :end-before: /privacy-loss

3. Collect Public Information
-----------------------------

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: public-info
            :end-before: /public-info

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: public-info
            :end-before: /public-info

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: public-info
            :end-before: /public-info

4. Mediate Access to Data
-------------------------

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: mediate
            :end-before: /mediate

        ``dp.Context.compositor`` creates an adaptive composition measurement.
        You can now submit up to three queries to ``context``, in the form of measurements.

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: mediate
            :end-before: /mediate

        ``dp.c.make_adaptive_composition`` creates an adaptive composition measurement.
        You can now submit up to three queries to ``queryable``, in the form of :ref:`Measurements <measurements-user-guide>`.

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: mediate
            :end-before: /mediate

        ``make_adaptive_composition`` creates an adaptive composition measurement.
        You can now submit up to three queries to ``queryable``, in the form of measurements.

5. Submit DP Queries
--------------------

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: mean
            :end-before: /mean

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: mean
            :end-before: /mean

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: mean
            :end-before: /mean