.. _contribution-process:

Contribution Process
********************

This section documents how to contribute your changes to the `OpenDP Library <https://github.com/opendp/opendp/>`_.

Contributor License Agreement
-----------------------------

In anticipation of making a contribution, please be aware that the OpenDP Project requires contributors to sign
a Contributor License Agreement.
We use a bot for automated verification of license status, so your first OpenDP pull request will be tagged with
instructions for completing the agreement.
If you'd like to get a head start, you can view our :ref:`cla` documentation.

Creating a Pull Request
-----------------------
A pull request is a request to have your changes "pulled" into the OpenDP codebase.

* `Creating a pull request <https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request>`_
* `Creating a pull request from a fork <https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request-from-a-fork>`_

The branch naming convention is the issue number followed by a short description beginning with a verb.
For example: ``123-fix-typo-in-readme``.

Github will name the PR from this branch ``123 fix typo in readme``; Retitle to ``Fix typo in readme``.
(Our script groups changelog entries by the first word of the PR title, so these name conventions help.)

Once you have opened your Pull Request, it will appear in the `list of open PRs <https://github.com/opendp/opendp/pulls>`_.
Please check back later for feedback!

Branch Philosophy
-----------------
OpenDP may have many open PRs at any given moment.
Sometimes it's just a backlog, but also reflects our development process:

When possible, we want PRs to be narrowly focused, and linked to a single issue.

But bigger features will have multiple prerequisites...
and there may be different ways of satisfying those prerequisites,
and it may not be obvious at first what the best course is.
Rather than merging those prerequisite PRs straight to ``main`` we'll stack other PRs on them,
and those later PRs can test whether we made the right choices at the beginning.
Use-case partners are also using their own forks of OpenDP in production,
and this in-the-field experience leads to improvements in PRs,
but it will take some time to integrate their contributions.
Finally, we use `graphite <https://graphite.dev/>`_, which makes it easier to handle PR stacks.

So, while we often do have a large number of open PRs, it is part of a process that has worked for our team,
and it has given us the time to reflect on issues in algorithms and software engineering,
and still make regular releases.


Vetting Process
---------------
A contribution can be reviewed and merged, with ``contrib`` flags set, without completing the vetting process.
``contrib`` components are disabled by default, but an end-user can choose to opt-in.

On the OpenDP side, the vetting process to remove the ``contrib`` flag goes as follows:

#. Reviewers are assigned.
#. Reviewer confirms the privacy claims of the proof.
#. Reviewer validates the pseudo-code against the proof.
#. Reviewer validates the code against the pseudo-code.

Please be patient— this process may take several iterations as issues come up in the review process.

Merge
-----

Once the review process is successful, your PR will be merged into the ``main`` branch.
Your contribution and credit will be added to the release notes,
and your changes will appear on crates.io and PyPi on our next library release.
