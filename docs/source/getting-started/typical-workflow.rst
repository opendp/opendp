Typical Workflow
================

A differentially private analysis in OpenDP typically has the following steps:

1. Identify the unit of privacy
2. Set privacy loss parameters
3. Mediate access to data
4. Submit DP queries (examples: count and sum)

We'll illustrate these steps by releasing a differentially private mean of a small vector of random numbers.

1. Identify the Unit of Privacy
-------------------------------

The first step in a differentially private analysis is to determine what you are protecting: the unit of privacy.

Releases on the data should conceal the addition or removal of any one individual's data.
Assuming you know an individual may contribute at most one row to the data set, 
then the unit of privacy corresponds to one row contribution.

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

The privacy unit specifies how distances are computed between two data sets (``input_metric``), and how large the distance can be (``d_in``).

Broadly speaking, differential privacy can be applied to any medium of data for which you can define a unit of privacy. 
In other contexts, the unit of privacy may correspond to multiple rows, a user ID, or nodes or edges in a graph.

The unit of privacy may also be more general or more precise than a single individual.

* *more general*: unit of privacy is an entire household, or a company
* *more precise*: unit of privacy is a person-month, or device

It is highly recommended to choose a unit of privacy that is at least as general as an individual.

2. Set Privacy Loss Parameters
------------------------------

Next, you should determine what level of privacy protection to provide to your units of privacy. 
This choice may be governed by a variety of factors, 
such as the amount of harm that individuals could experience if their data were revealed, 
and your ethical and legal obligations as a data custodian.

The level of privacy afforded to units of privacy in a data set is quantified by *privacy loss parameters*. 
Under *pure* differential privacy, there is a single privacy-loss parameter, typically denoted epsilon (ε). 
Epsilon is a non-negative number, where larger values afford less privacy. 
Epsilon can be viewed as a proxy for the worst-case risk to a unit of privacy. 
It is customary to refer to a data release with such bounded risk as epsilon-differentially private (ε-DP).

A common rule-of-thumb is to limit ε to 1.0, but this limit will vary depending on the considerations mentioned above. 
See the `Deployments Registry <http://registry.opendp.org/deployments-registry/>`_ for examples of parameters used by real-world applications.

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

The privacy loss consists of how distances are measured between distributions (``privacy_measure``), and how large the distance can be (``d_out``).

3. Mediate Access to Data
-------------------------

Ideally, at this point, you have not yet accessed the sensitive data set. 
This is the only point in the process where we access the sensitive data set. 
To ensure that your specified differential privacy protections are maintained, 
the OpenDP Library should mediate all access to the sensitive data set.

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: mediate
            :end-before: /mediate

        ``dp.Context.compositor`` creates a fully adaptive composition measurement.
        You can now submit any number of queries to ``context``,
        so long as the privacy loss doesn't exceed ``d_out``.

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: mediate
            :end-before: /mediate

        ``o_ac`` is a fully-adaptive composition odometer, *without* a limit on the privacy loss.
        ``m_ac`` is a fully-adaptive composition measurement, *with* a limit on the privacy loss.
        You can now submit any number of queries to ``queryable``, in the form of measurements,
        so long as the privacy loss doesn't exceed ``d_out``.

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: mediate
            :end-before: /mediate

        ``o_ac`` is a fully-adaptive composition odometer, *without* limit on the privacy loss.
        ``m_ac`` is a fully-adaptive composition measurement, *with* a limit on the privacy loss.
        You can now submit any number of queries to ``queryable``, in the form of measurements,
        so long as the privacy loss doesn't exceed ``d_out``.

A good practice at this point is to drop the data so that you aren't tempted to access it.

4. Submit DP Queries
--------------------

You can now create differentially private releases.
Here's a differentially private count, 
with an accuracy estimate and confidence interval:

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: count
            :end-before: /count

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: count
            :end-before: /count

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: count
            :end-before: /count

This differentially private mechanism simply adds discrete laplace noise to the exact count.

To compute a differentially private sum, we'll need to bound the range of each data record
to ensure that the influence each record has on the sum is bounded.
The choice of bounds must be *public information*.

Public information includes:

* Information that is publicly available from other sources
* Information from other DP releases

In this case, if you know the data represents ages of individuals,
you can make a reasonable guess as to what good data bounds would be.

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

To avoid introducing bias in your estimate, 
your choice of bounds should include most of the values in the data.
At the same time, the wider the bounds are, the greater the variance the estimate will be,
because more noise will be necessary to satisfy the privacy guarantee.

.. tab-set::

    .. tab-item:: Context API (Python)
        :sync: context

        .. literalinclude:: code/typical-workflow-context.rst
            :language: python
            :dedent:
            :start-after: sum
            :end-before: /sum

    .. tab-item:: Framework API (Python)
        :sync: framework

        .. literalinclude:: code/typical-workflow-framework.rst
            :language: python
            :dedent:
            :start-after: sum
            :end-before: /sum

    .. tab-item:: Framework API (R)
        :sync: r

        .. literalinclude:: code/typical-workflow-framework.R
            :language: r
            :start-after: sum
            :end-before: /sum

Altogether, the data flow of this analysis looks like:

.. Diagram source: https://docs.google.com/drawings/d/1W4l9x3UM3hbVLWlC0nzijqgaQ31wY5ERebp8jkYy1yc/edit

.. image:: code/typical-workflow-diagram.svg
    :width: 100%
    :alt: Diagram representing typical data flow with OpenDP, raw data to differentially private releases. 

The plugin estimates can then be used together to estimate the mean.

The following sections cover APIs for building differentially private queries.
The OpenDP Library supports a more idiomatic API for working with tabular data through Polars,
approaches for statistical modeling in the style of scikit-learn,
as well as building-block mechanisms that can be embedded in your own data pipelines.
