:orphan:
:nosearch:

Working with Tabular Data
=========================
:orphan:
:nosearch:

Working with Tabular Data
=========================

Motivation 
----------

In the following examples, we will use real data to demonstrate the utility of OpenDP functions.
We will use the `Labour Force Survey microdata <https://ec.europa.eu/eurostat/web/microdata/public-microdata/labour-force-survey>`_ released by Eurostat for a few reasons: 
In the following examples, we will use real data to demonstrate the utility of OpenDP functions.
We will use the `Labour Force Survey microdata <https://ec.europa.eu/eurostat/web/microdata/public-microdata/labour-force-survey>`_ released by Eurostat for a few reasons: 

1. **Generality:** The dataset is relatively general, with variables and contexts accessible to users across various domains.
2. **Sample Utility:** The public microdata is a sample of the private, full microdata. Methods developed with the public microdata will also work on the private microdata, and researchers can request access to the full dataset through Eurostat. 
3. **Realism**: Since the dataset tracks individuals over multiple years, we need to think more carefully about our unit of privacy.

The specific methods that will be demonstrated are: 

* Computing Fundemental Statistics 
    * Sum 
    * Mean 
    * Median 
    * Quantiles 
* Aggregrations and Filtering 
    * Grouping By Singular Variables
    * Grouping By Multiple Variables 
    * Filtering 
2. **Sample Utility:** The public microdata is a sample of the private, full microdata. Methods developed with the public microdata will also work on the private microdata, and researchers can request access to the full dataset through Eurostat. 
3. **Realism**: Since the dataset tracks individuals over multiple years, we need to think more carefully about our unit of privacy.

Dataset Description 
-------------------

The data is organized by year and quarter for each nation in the European Union. For this tutorial, we sampled 200,000 individuals from the public microdata of France across all study years. 

The public microdata is protected using traditional statistical disclosure control methods such as global recoding, local suppression, and addition of noise. 
The public microdata is protected using traditional statistical disclosure control methods such as global recoding, local suppression, and addition of noise. 


Core Variables 
--------------
The `User Guide <https://ec.europa.eu/eurostat/documents/1978984/6037342/EULFS-Database-UserGuide.pdf>`_ describes many variables. Our examples will use just a few. (Descriptions are copied from the User Guide.) 
The `User Guide <https://ec.europa.eu/eurostat/documents/1978984/6037342/EULFS-Database-UserGuide.pdf>`_ describes many variables. Our examples will use just a few. (Descriptions are copied from the User Guide.) 

.. list-table:: 
   :header-rows: 1

   * - Variable
     - Definition
     - Coding
   * - ``SEX``
   * - ``SEX``
     - Sex
     - 1: Male; 2: Female
   * - ``AGE``
     - 1: Male; 2: Female
   * - ``AGE``
     - Age of the Individual During the Reference Week
     - Single Years
   * - ``ILOSTAT``
   * - ``ILOSTAT``
     - Labour Status During the Reference Week
     - 1: Did any work for pay or profit during the reference week - one hour or more (including family workers but excluding conscripts on compulsory military or community service); 2: Was not working but had a job or business from which he/she was absent during the reference week (including family workers but excluding conscripts on compulsory military or community service); 3: Was not working because of lay-off; 4: Was a conscript on compulsory military or community service; 5: Other (15 years or more) who neither worked nor had a job or business during the reference week; 9: Not applicable (child less than 15 years old)
   * - ``HWUSUAL``
     - 1: Did any work for pay or profit during the reference week - one hour or more (including family workers but excluding conscripts on compulsory military or community service); 2: Was not working but had a job or business from which he/she was absent during the reference week (including family workers but excluding conscripts on compulsory military or community service); 3: Was not working because of lay-off; 4: Was a conscript on compulsory military or community service; 5: Other (15 years or more) who neither worked nor had a job or business during the reference week; 9: Not applicable (child less than 15 years old)
   * - ``HWUSUAL``
     - Number of Hours Per Week Usually Worked
     - 00: Usual hours cannot be given because hours worked vary considerably from week to week or from month to month; 01-98: Number of hours usually worked in the main job; 99: Not applicable; blank: No answer
   * - ``QUARTER``
     - 00: Usual hours cannot be given because hours worked vary considerably from week to week or from month to month; 01-98: Number of hours usually worked in the main job; 99: Not applicable; blank: No answer
   * - ``QUARTER``
     - Fixed Reference Quarter
     - Single Quarter
   * - ``YEAR``
   * - ``YEAR``
     - Fixed Reference Year
     - Single Year

Compositor Overview
-------------------
The compositor is the foundation of our differentially private queries. It essentially takes in our data and our specifications for the queries that we would like to run. At this point, we won't be directly referencing our data again and we could theoretically delete it! 

.. code-block:: python

   context = dp.Context.compositor(
       data=df,
       privacy_unit=dp.unit_of(contributions=36),
       privacy_loss=dp.loss_of(epsilon=1.0),
       split_evenly_over=10,
       margins={
           ("SEX", ): dp.Margin(max_partition_length=60_000_000),
           ("AGE", ): dp.Margin(max_partition_length=60_000_000),
           ("ILOSTAT", ): dp.Margin(max_partition_length=60_000_000),
           ("HWUSUAL", ): dp.Margin(max_partition_length=60_000_000),
           ("YEAR", ): dp.Margin(max_partition_length=60_000_000, max_partition_contributions=4),
           ("QUARTER", ): dp.Margin(max_partition_length=60_000_000, max_partition_contributions=13),
           ("YEAR", "QUARTER",): dp.Margin(max_partition_length=60_000_000, max_partition_contributions=1),
           (): dp.Margin(max_partition_length=60_000_000),
       },
   )

**Parameters**:

* *privacy_unit:* How many rows each individual or entity of interest contributes to our data frame. In this case, we are analyzing the data from across 13 years and each year has 4 quarters. Therefore, the unit of privacy is 36. If we were to analyze a particular quarter in a particular year, the unit of privacy would be 1 since each individual would be represented once. 

* *privacy_loss:* This parameter determines how much privacy we want to preserve. If ε is small, we will have more privacy but worse data accuracy. ε can range from 0 to infinity, but 1 is usually a standard. 

* *split_evenly_over:* This is the number of queries you want to distribute your privacy loss over. For now we specified 10 to explore the API but in the final versions of your code, this parameter will be picked more carefully. 

* *margins:* Margins capture the variables of interest in your analysis. This can include variables that you may want to group by or apply differential privacy techniques to. 
    * *max_partition_length:* The upper bound on how many records (individuals in this case) can be in one partition. If you do not know the size of your dataset, this can be an upper bound on the population represented in your dataset. The population of France was about 60 million in 2004 so that's our maximum partition length. Source: `World Bank <https://datatopics.worldbank.org/world-development-indicators/>`_. 
    * *max_partition_contributions:* The number of contributions each individual can have per grouping. Since each individual is represented once for a particular quarter and year, they are represented 13 times for each quarter since there are 13 years in the dataset and 4 times each year since there are 4 quarters within a year. 
    * *max_partition_length:* The upper bound on how many records (individuals in this case) can be in one partition. If you do not know the size of your dataset, this can be an upper bound on the population represented in your dataset. The population of France was about 60 million in 2004 so that's our maximum partition length. Source: `World Bank <https://datatopics.worldbank.org/world-development-indicators/>`_. 
    * *max_partition_contributions:* The number of contributions each individual can have per grouping. Since each individual is represented once for a particular quarter and year, they are represented 13 times for each quarter since there are 13 years in the dataset and 4 times each year since there are 4 quarters within a year. 

Particular examples will require additional parameters, and the compositor will change slightly.
See :py:func:`opendp.context.Context.compositor` for more information.
Particular examples will require additional parameters, and the compositor will change slightly.
See :py:func:`opendp.context.Context.compositor` for more information.