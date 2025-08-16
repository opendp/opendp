.. _tabular-data:

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

We will use a `Labour Force Survey microdata <https://ec.europa.eu/eurostat/web/microdata/public-microdata/labour-force-survey>`_ released by Eurostat
for this tutorial, with some `additional preprocessing <https://github.com/opendp/dp-test-datasets/blob/main/data/eurostat/README.ipynb>`__.
On a quarterly cadence Eurostat surveys the working hours of individuals in the European Union.
The public microdata is protected using traditional statistical disclosure control methods such as global recoding, local suppression, and addition of noise. 

We chose this dataset for a few reasons: 

1. **Accessibility:** The dataset is accessible to users across various domains.
2. **Sample Utility:** The public microdata is a sample of the private, full microdata. Methods developed with the public microdata will also work on the private microdata, and researchers can request access to the full dataset through Eurostat. 
3. **Realism**: This is a real dataset that tracks individuals over multiple years, which will influence the unit of privacy since each individual can be represented multiple times in the dataset. 

For this tutorial, we selected a few columns of interest from the public microdata of France across 9 study years. 

The `User Guide <https://www.gesis.org/missy/files/documents/EU-LFS/EULFS_Database_UserGuide_2021-3.pdf>`_
for the dataset describes many variables. 
Our examples will use just a few. (Descriptions are copied from the User Guide.) 

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

While the dataset does not contain a unique identifier for individuals,
we've generated a synthetic column of unique identifiers, ``PIDENT``, for the purpose of demonstrating library functionality.


Mediate access with ``Context``
-------------------------------

The ``Context`` is the foundation of our differentially private analysis. 
It mediates access to the sensitive data,
ensuring that queries you would like to release satisfy necessary privacy properties. 

.. tab-set::

    .. tab-item:: Python
        :sync: python

        .. code:: python
          
            >>> import polars as pl
            >>> df = pl.LazyFrame()

            >>> context = dp.Context.compositor(
            ...     data=df,
            ...     privacy_unit=dp.unit_of(contributions=36),
            ...     privacy_loss=dp.loss_of(epsilon=1.0),
            ...     split_evenly_over=1,
            ... )
            
            >>> # Once you construct the context, you should abstain
            >>> # from directly accessing your data again.
            >>> # In fact, it is good practice to delete it! 
            >>> del df

A number of parameters define the ``Context``:

The ``privacy_unit`` describes the greatest influence one individual may have on your dataset.
In this case we have quarterly data across 9 years, and an individual can contribute up to one row per quarter,
so an individual can contribute up to 36 rows. 
If we were to analyze a particular quarter in a particular year,
the unit of privacy would be 1 since each individual would contribute at most one row.   
(Later in this tutorial we'll make this simpler by using the identifier column ``PIDENT`` 
to define the privacy unit.)

The ``privacy_loss`` is the greatest privacy loss an individual in the dataset can experience.
There are different models of differential privacy.
Pure DP, used here, has only a single parameter, epsilon (ε).
Later in the documentation we'll introduce other models of differential privacy
with other parameters. These other models give us flexibility and may help us use our
privacy budget more efficiently.

Finally, how many queries can we make?
Placing a limit on the number of queries is essential if we want to bound privacy loss.
There are two options: ``split_evenly_over`` is best if your queries are all similar.
Alternatively, ``split_by_weights`` lets you give more of your budget to more important queries.

Later examples in this tutorial will also introduce the idea of "margins".

See the Polars section in the `OpenDP User Guide <../../api/user-guide/polars/index.html>`_
for more information on any of these topics.

.. toctree::
  :maxdepth: 1

  essential-statistics
  grouping
  identifier-truncation
  preparing-microdata