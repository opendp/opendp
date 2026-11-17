Welcome
=======

What is differential privacy?
-----------------------------

Differential privacy is a rigorous mathematical definition of privacy.
Consider an algorithm that analyzes a dataset and releases statistics:
The algorithm is differentially private if by looking at the output,
you cannot tell whether any individual's data was included in the original dataset or not.
Differential privacy achieves this by carefully injecting random noise into the released statistics to hide the effects of each individual. 

For more background on differential privacy and its applications:

* "`Designing Access with Differential Privacy <https://admindatahandbook.mit.edu/book/v1.0/diffpriv.html>`_" from *Handbook on Using Administrative Data for Research and Evidence-based Policy*
* The `Resources list <https://differentialprivacy.org/resources/>`_ from *differentialprivacy.org*
* A shorter list of `Educational Resources <https://opendp.github.io/learning/>`_
* OpenDP's own :doc:`theory/resources`
* `Hand-On Differential Privacy <https://www.oreilly.com/library/view/hands-on-differential-privacy/9781492097730/>`_ is co-authored by the OpenDP Library architect

Why OpenDP?
-----------

* OpenDP is based on a `solid conceptual framework <https://opendp.org/files/2025/11/opendp_programming_framework_11may2020_1_01.pdf>`_ for expressing privacy-aware computations.
* OpenDP is built on a Rust core for memory and thread safety and performance.
* OpenDP has a process for independent review of algorithms and implementations.
* OpenDP has performed well in `independent security audits <https://www.sri.inf.ethz.ch/publications/lokna2023groupandattack>`_.
* OpenDP supports a range of differential privacy algorithms.
* OpenDP has bindings for Python and R, both built on the same Rust core for consistency and security.
* OpenDP is a community effort and is not owned or directed by a single corporation.

That said, OpenDP is not the best tool for every job.
In particular, it is a fairly low-level interface:
There are a number of other projects which try to make it easy to add
differential privacy to existing SQL interfaces or ML frameworks.
One such tool is `SmartNoise SDK <https://github.com/opendp/smartnoise-sdk>`_,
which is built on the OpenDP library.

Who is using OpenDP?
--------------------

Some of the applications of OpenDP in healthcare, government, and tech include:

* The `UN High Commissioner for Refugees (UNHCR) <https://www.unhcr.org>`_ used OpenDP to explore the creation of `synthetic microdata for refugee populations <https://microdata.unhcr.org/index.php/synthetic-data>`_.
* The `Swiss Federal Statistical Office <https://www.bfs.admin.ch/bfs/en/home.html>`_ used OpenDP to prototype tools to `release income statistics <https://www.bfs.admin.ch/bfs/en/home/dscc/blog/2024-02-opendp.html>`_ and has built `Lomas <https://github.com/dscc-admin-ch/lomas>`_, a platform for remote data science that integrates with OpenDP.
* `Oblivious <https://www.oblivious.com>`_ used OpenDP for major U.N. and telecom pilot projects as part of their `Antigranular <https://docs.antigranular.com/private-python/packages/opendp/>`_ data privacy platform.
* `OpenMined <https://openmined.org>`_ used OpenDP as part of PySyft deployments with Microsoft and DailyMotion, with pilots at multiple national statistical organizations.
* Researchers at `Microsoft <https://microsoft.com>`_ used OpenDP to construct a `United States Broadband Percentages Database <https://github.com/microsoft/USBroadbandUsagePercentages>`_.
* Researchers at `Harvard Business School <https://www.hbs.edu/>`_ and `Microsoft <https://microsoft.com>`_ used OpenDP to create `a detailed atlas of internet usage in the US <https://www.nber.org/papers/w32932>`_ and found substantial disparities between urban and rural areas, and even within cities.
* `LiveRamp <https://liveramp.com>`_ used OpenDP to `support COVID research <https://liveramp.uk/developers/blog/two-liveramp-engineers-named-harvard-opendp-fellows/>`_.
* Researchers with the `Christchurch Call Initiative on Algorithmic Outcomes <https://www.christchurchcall.org/christchurch-call-initiative-on-algorithmic-outcomes/>`_ used OpenDP to `audit recommender algorithms <https://www.christchurchcall.org/content/files/2024/11/Christchurch-Call-AI-Transparency-in-Practice-Report-October-2024-1.pdf>`_.

Let us know if you have an example to add!

What next?
----------

There are multiple tracks through the documentation:

* New users of the library should begin with :doc:`getting-started/index`.
* For Python, R, and Rust references, see the :doc:`api/index`.
* If you want to understand how the fundamentals of DP are applied in OpenDP, see :doc:`theory/index`.
* Finally, if you're joining the project, see :doc:`contributing/index`.

.. toctree::
  :hidden:

  getting-started/index
  api/index
  theory/index
  contributing/index
