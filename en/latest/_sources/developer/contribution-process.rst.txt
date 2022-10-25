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

The branch naming convention is the issue number followed by a short description, such as ``123-typo-in-readme``.
Once you have opened your Pull Request, it will appear in the `list of open PRs <https://github.com/opendp/opendp/pulls>`_.
Please check back later for feedback!


Vetting Process
---------------
A contribution can be reviewed and merged, with ``contrib`` flags set, without completing the vetting process.
``contrib`` components are disabled by default, but an end-user can choose to opt-in.

On the OpenDP side, the vetting process to remove the ``contrib`` flag goes as follows:

#. Reviewers are assigned.
#. Reviewer confirms the privacy claims of the proof.
#. Reviewer validates the pseudo-code against the proof.
#. Reviewer validates the Rust code against the pseudo-code.

Please be patient— this process may take several iterations as issues come up in the review process.

Merge
-----

Once the review process is successful, your PR will be merged into the ``main`` branch.
Your contribution and credit will be added to the release notes,
and your changes will appear on crates.io and PyPi on our next library release.
