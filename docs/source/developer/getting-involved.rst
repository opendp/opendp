.. _getting-involved:

Getting Involved
================

OpenDP is a community effort and we welcome your contributions!
OpenDP development takes place on GitHub, so you will need to create a GitHub account to get started.

Bug Reports
-----------
Email security@opendp.org to report a security issue.

Please `open a new issue <https://github.com/opendp/opendp/issues/new?template=bug-report.md>`__ for any other bug report.
We also welcome new issues for usability problems!
We want to know if you've encountered, say, a missing piece of documentation, or a particularly confusing interface.

Code and Proof Contributions
----------------------------
Please `open a new issue <https://github.com/opendp/opendp/issues/new?template=new-contribution.md>`__ for any new code or proof contribution.

We would like to have an opportunity to talk with you and make design suggestions before you invest significant effort in your contribution.
Opening an issue first also gives us a chance to connect you with people who are doing similar work
or identify existing tooling that may be useful to you.

If you open an issue for adding a new constructor,
please include a link to the research paper(s) your contribution will be derived from,
if you already have an existing implementation,
and make a case for why this constructor should be added to the library.

* Is it unique and does it have reasonable utility?
* If it has a similar behavior as another constructor, can you show that it has greater utility or useful trade-offs?


Resolve an Issue
----------------
Take a look through `the issue board <https://github.com/opendp/opendp/issues>`_ if you do not already have a specific contribution in mind.

Once you find an issue you can tackle, comment on the issue to claim it.
Feel free to ask clarifying questions!


Write Documentation
-------------------
We need your help to make OpenDP more accessible through documentation.
There are many forms of documentation!

* Python docstrings and type hints
* Long-form sphinx documentation `located here <https://github.com/opendp/opendp/tree/main/docs>`_
* Expanding `Rustdocs <https://docs.rs/opendp/0.2.1/opendp/>`_ on core library modules (`example <https://github.com/opendp/opendp/blob/main/rust/src/lib.rs#L1>`_), structures and constructors
* Notebooks and example applications

Please `open a new issue <https://github.com/opendp/opendp/issues/new?template=new-contribution.md>`__ to take ownership of a piece of documentation.


Add Tests
---------
We need your help to make OpenDP more stable by adding tests.

* Python: `pytest unit and integration tests <https://github.com/opendp/opendp/tree/main/python/test>`_
* Rust: in-file testing modules (`core example <https://github.com/opendp/opendp/blob/ead11c5dbadfb17062182da6799f400888e66cef/rust/opendp/src/trans/count/mod.rs#L121-L182>`_, `ffi example <https://github.com/opendp/opendp/blob/ead11c5dbadfb17062182da6799f400888e66cef/rust/opendp-ffi/src/trans/resize.rs#L53-L93>`_)

All tests (even ones embedded inside docstrings and this docs site) are checked automatically via CI.

There are also classes of tests that we are completely missing:

* performance benchmarking for regressions and timing attacks
* tests for utility
* applying stochastic testers to relations

Please `open a new issue <https://github.com/opendp/opendp/issues/new?template=new-contribution.md>`__ to communicate what you are planning to work on.


Review Pull Requests
--------------------
It always helps to have another set of eyes reviewing code.
This can also be a great way to familiarize yourself with the library internals and development process.

Respond to Discussion Posts
---------------------------
We want to develop `discussions into a useful community forum <https://github.com/opendp/opendp/discussions>`_.
Sharing your experience on this public forum would, of course, help make differential privacy more accessible!
