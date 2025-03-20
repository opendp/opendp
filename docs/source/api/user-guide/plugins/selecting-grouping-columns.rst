Privately Selecting Grouping Columns
====================================

This longer example demonstrates how plugins could be used on a real-world problem.

Imagine you want to group the rows in a private dataset before releasing aggregate statistics,
but you yourself are not allowed to look at the private data, and you don't know what columns to group by.
You do know that there are three types of columns:

* Columns that are too uniform: most rows have the same value.
* Columns that are too diverse: most rows have unique values.
* Columns that are just right

If you group by too many columns,
then the number of individuals contributing to each combination of attributes will be small,
resulting in most combinations being censored in the final release.
On the other hand, you want granular statistics, so more grouping columns is appealing.

The example below demonstrates how to construct your own mechanism that chooses a set of grouping columns for you.
It also makes use of library plugins (via a user-defined transformation and domain)
and the Report Noisy Max Gumbel mechanism.

We'll first write plugins for our transformation and measurement:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/selecting-grouping-columns.txt
            :language: python
            :start-after: plugins
            :end-before: /plugins

Next, use these functions to create a DP mechanism:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/selecting-grouping-columns.txt
            :language: python
            :start-after: dp-mechanism
            :end-before: /dp-mechanism

Finally, load your data and make a DP release:

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. literalinclude:: code/selecting-grouping-columns.txt
            :language: python
            :start-after: dp-release
            :end-before: /dp-release

Successive runs will return different sets of columns that satisfy your criteria.
