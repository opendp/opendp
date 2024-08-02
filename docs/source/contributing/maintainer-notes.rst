Maintainer Notes
****************

This section maps out development logistics for the core team of OpenDP developers.

.. I left this here because it's actually deeply nested

.. contents:: |toctitle|
    :local:

Summary
=======

* Base all code on a long-lived branch called ``main``, where we enforce a linear history.
* Do development on short-lived feature branches, which are merged to ``main`` with squash commits.
* Generate new versions via GitHub Releases, using long-lived release branches.

Rationale
---------

Our process should be as simple as feasible -- but not simpler!
We need to balance developer friendliness with the special requirements of privacy-sensitive software.

We want a clean ``main`` branch that is always in a known good state.
It should always be usable for development, and all tests should pass.
This allows us to create releases easily, using the most up-to-date code.

We need linear code history, without unanticipated changes introduced by merge commits.
This is important so that contributions are validated using the exact code that will land on ``main``.

We need separate release branches to allow bug fixes for previous versions.
This is important to support library users who can't upgrade to the latest version of OpenDP yet.

All release tasks should be automated and drivable from GitHub.
Creating a release should not require setup of a local environment, and should not have any dependencies on non-standard tools.
This is important to allow for delegation of tasks and continuity of the project.


Task Tracking
=============

* Use GitHub Issues to track all tasks. This is helpful to know who's working on what.
* Use the `OpenDP Development GitHub Project <https://github.com/orgs/opendp/projects/1?card_filter_query=label%3A%22opendp+core%22>`_
  to organize work and prioritize tasks for development.
* Manage all changes using GitHub Pull Requests.

Code Hygiene
============

* Follow the Rust guidelines for coding style. Code should be formatted using the default settings of rustfmt.
  (TODO: Automatic marking of style issues on PRs -- https://github.com/opendp/opendp/issues/256)
* Write API docs for all public and significant private APIs.
  We use inline comments to explain complicated code.
* Make sure ``main`` is always in a good state: Code compiles, tests pass.
  If ``main`` is ever broken, it should be the team's top priority to fix it.
* Use GitHub Actions to check PRs automatically, and don't allow merges if checks fail.

Branching Strategy
==================

* Use a single, long-lived branch named ``main`` for core project history.
* Maintain a linear history on ``main`` and a 1-to-1 correspondence between PRs and commits by using squash merges.
* Feature branches have the naming scheme ``<nnn>-<short-desc>``,
  where ``<nnn>`` is the number of the GitHub issue tracking this task,
  and ``<short-desc>`` is a short description of the change. For instance, ``123-new-measurement``
* Manage all changes using GitHub PRs.
  Check for test success and do code reviews before approving PRs.
* Try to keep PRs relatively small, self-contained, and focussed on a single issue.
  This simplifies code reviews, and reduces the likelihood of rebasing hassles.
* To enforce this strategy, use the following branch protections on main:

  * Require pull request reviews before merging
  * Require approvals
  * Dismiss stale pull request approvals when new commits are pushed
  * Require status checks to pass before merging
  * Require linear history


Release Workflows
-----------------

Our `release process <https://github.com/opendp/opendp/tree/main/.github/workflows#making-a-release>`_
uses github workflows.
