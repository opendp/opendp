Working with Tabular Data
=========================


OpenDP uses `Polars <https://pola.rs/>`_ to work with dataframes.
This functionality is not enabled by default.


.. tab-set::

    .. tab-item:: Python
        :sync: python

        Add ``[polars]`` to the package installation to enable Polars functionality.

        .. prompt:: bash

            pip install 'opendp[polars]'

        This installs the specific version of the Python Polars package that is compatible with the OpenDP Library.

    .. tab-item:: R
        :sync: r

        OpenDP does not currently support Polars in R. 
        For the current status of this, see `issue #1872 <https://github.com/opendp/opendp/issues/1872>`_.

    .. tab-item:: Rust
        :sync: rust

        Add ``"polars"`` to the features of the ``opendp`` dependency in your ``Cargo.toml``.


Dataset Description 
-------------------

For this section of the documentation, we will use the `Labour Force Survey microdata <https://ec.europa.eu/eurostat/web/microdata/public-microdata/labour-force-survey>`_ released by Eurostat.
The data surveys working hours of individuals in the European Union collected on a quarterly cadence.
The public microdata is protected using traditional statistical disclosure control methods such as global recoding, local suppression, and addition of noise. 

We chose this dataset for a few reasons: 

1. **Accessibility:** The dataset is accessible to users across various domains.
2. **Sample Utility:** The public microdata is a sample of the private, full microdata. Methods developed with the public microdata will also work on the private microdata, and researchers can request access to the full dataset through Eurostat. 
3. **Realism**: This is a real dataset that tracks individuals over multiple years, which will influence the unit of privacy since each individual can be represented multiple times in the dataset. 

For this tutorial, we sampled a total of 200,000 rows from the public microdata of France across all study years. 

The `User Guide <https://ec.europa.eu/eurostat/documents/1978984/6037342/EULFS-Database-UserGuide.pdf>`_ describes many variables. Our examples will use just a few. (Descriptions are copied from the User Guide.) 

.. list-table:: 
   :header-rows: 1

   * - Variable
     - Definition
     - Coding
   * - ``SEX``
     - Sex
     - | ``1``: Male
       | ``2``: Female
   * - ``AGE``
     - Age of the Individual During the Reference Week
     - Single Years
   * - ``ILOSTAT``
     - Labour Status During the Reference Week
     - | ``1``: Did any work for pay or profit during the reference week - one hour or more (including family workers but excluding conscripts on compulsory military or community service)
       | ``2``: Was not working but had a job or business from which he/she was absent during the reference week (including family workers but excluding conscripts on compulsory military or community service)
       | ``3``: Was not working because of lay-off
       | ``4``: Was a conscript on compulsory military or community service
       | ``5``: Other (15 years or more) who neither worked nor had a job or business during the reference week
       | ``9``: Not applicable (child less than 15 years old)
   * - ``HWUSUAL``
     - Number of Hours Per Week Usually Worked
     - | ``00``: Usual hours cannot be given because hours worked vary considerably from week to week or from month to month
       | ``01`` - ``98``: Number of hours usually worked in the main job
       | ``99``: Not applicable
       | *blank*: No answer
   * - ``QUARTER``
     - Fixed Reference Quarter
     - Single Quarter
   * - ``YEAR``
     - Fixed Reference Year
     - Single Year


Compositor Overview
-------------------

The compositor is the foundation of our differentially private analysis. 
It mediates access to the sensitive data,
ensuring that queries you would like to release satisfy necessary privacy properties. 

.. testsetup::
    >>> import polars as pl
    >>> df = pl.LazyFrame()

.. doctest:: python

    >>> context = dp.Context.compositor(
    ...     data=df,
    ...     privacy_unit=dp.unit_of(contributions=36),
    ...     privacy_loss=dp.loss_of(epsilon=1.0),
    ...     split_evenly_over=10,
    ... )
    
    >>> # Once you construct the context, you should abstain from directly accessing your data again.
    >>> # In fact, it is good practice to delete it! 
    >>> del df

Context Parameters
~~~~~~~~~~~~~~~~~~

* ``privacy_unit``: The greatest influence an individual may have on your dataset.
  In this case, the influence is measured in terms of the number of rows an individual may contribute to your dataset. 
  Since we are analyzing quarterly data across 9 years, where an individual contributes up to one record per quarter,
  the unit of privacy corresponds to 36 row contributions. 
  If we were to analyze a particular quarter in a particular year, the unit of privacy would be 1 since each individual would contribute at most one row. 
* ``privacy_loss``: The greatest privacy loss suffered by an individual in your dataset. 
  The privacy loss is upper-bounded by privacy parameters; in this case epsilon (ε).
* ``split_evenly_over``: This is the number of queries you want to distribute your privacy loss over. 
  Configure this parameter appropriately according to how many queries you would like to release. 

Particular examples in the coming sections may require additional parameters, 
and parameters to the compositor may be adjusted slightly.
See :py:func:`opendp.context.Context.compositor` for more information.


.. toctree::

  essential-statistics
  grouping