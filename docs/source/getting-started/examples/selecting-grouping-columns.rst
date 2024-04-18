Privately Selecting Grouping Columns
====================================

Imagine you have a tabular dataset with four columns:
``date``, ``merchant_postal_code``, ``merch_category`` and ``transaction_type``.
The data is sparse – not all combinations of these categories are present in the data.
To create a differentially private release with statistics grouped by these columns,
you can only release statistics for combinations of these attributes that many people contribute to.

If you group by too many columns,
then the number of individuals contributing to each combination of attributes will be small,
resulting in most combinations being censored in the final release.
On the other hand, you want granular statistics, so more grouping columns is appealing.

The example below demonstrates how to construct your own mechanism that chooses a set of grouping columns for you.
It also makes use of library plugins (via a user-defined transformation and domain)
and the Report Noisy Max Gumbel mechanism.

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/selecting-grouping-columns.rst
            :language: python
            :start-after: example
            :end-before: /example