Real World
==========

In this example, we'll conduct a differentially-private analysis on a teacher survey (a tabular dataset).

The raw data consists of survey responses from teachers in primary and secondary schools in an unspecified U.S. state.

TODO: Add note about provenance -- https://github.com/opendp/opendp/issues/1076

A differentially private analysis is usually conducted in the following steps:

1. Identify the unit of privacy
2. Consider privacy risk 
3. Collect public information
4. Construct a measurement
5. Make a DP release

It is common to return to prior steps to make further releases.

Identify the Unit of Privacy
----------------------------

We first need to isolate exactly what we're protecting.

In the teacher survey, the unit of privacy is the addition or removal of one teacher. 
Since each teacher contributes at most one row to the dataset, 
the unit of privacy corresponds to defining the maximum number of row contributions to be one.

.. doctest::

    >>> max_contributions = 1

We will use this bound to tune our methods such that data releases are quantifiably indistinguishable 
upon the addition or removal of any one teacher from any input dataset.

Broadly speaking, differential privacy can be applied to any medium of data for which you can define a unit of privacy.
In other contexts, the unit of privacy may correspond to multiple rows, a user ID, or nodes or edges in a graph.

The unit of privacy may also be more general or more precise than a single individual.
In the data analysis conducted in this notebook, we'll refer to an individual and the unit of privacy interchangeably, 
because in this example we've defined the unit of privacy to be one individual. 

Consider privacy risk
---------------------

The next step is to consider the risk of disclosing information about your sensitive dataset.

If the dataset is available to the public, then differentially private methods are not necessary.
Whereas, if the dataset could place individuals at severe risk of harm, then you should reconsider making any data release at all.
Differentially private methods are used to make releases on data with a risk profile somewhere between these two extremes.

The level of privacy afforded to individuals in a dataset under analysis is quantified by `privacy parameters`.
One such privacy parameter is epsilon (ε), a non-negative number, where larger values afford less privacy.
ε can be viewed as a proxy to quantify the worst-case risk to any individual.
It is customary to refer to a data release with such bounded risk as ε-DP.

A common rule-of-thumb is to limit your overall ε spend to 1.0.
However, this limit will vary depending on the risk profile associated with the disclosure of information.
In many cases, the privacy parameters are not finalized until the data owner is preparing to make a disclosure.


Collect Public Information
--------------------------

The next step is to identify public information about the dataset.

* information that is invariant across all potential input datasets (may include column names and per-column categories)
* information that is publicly available from other sources
* information from other DP releases

In this case (and in most cases), we consider column names public/invariant to the data because they weren't picked in response to the data, they were "fixed" before collecting the data.

A data invariant is information about your dataset that you are explicitly choosing not to protect,
under the basis that it does not contain sensitive information. 
Be careful because, if an invariant does, indeed, contain sensitive information,
then you expose individuals in the dataset to unbounded privacy loss.

This public metadata will significantly improve the utility of our results.


Construct a Measurement
-----------------------

TODO


Make a DP Release
-----------------

TODO


