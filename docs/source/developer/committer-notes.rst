Committer Notes
***************

This section maps out development logistics for core team of OpenDP developers.

.. contents:: |toctitle|
    :local:

Summary
=======

* Base all code on a long-lived branch called ``main``, where we enforce a linear history.
* Do development on short-lived feature branches, which are merged to ``main`` with squash commits.
* Generate new versions via GitHub Releases, using long-lived release branches.

Rationale
---------

Our process should be as simple as feasible -- but not simpler! We need to balance developer friendliness with the special requirements of privacy-sensitive software.

We want a clean ``main`` branch that is always in a known good state. It should always be usable for development, and all tests should pass. This allows us to create releases easily, using the most up-to-date code.

We need linear code history, without unanticipated changes introduced by merge commits. This is important so that contributions are validated using the exact code that will land on ``main``.

We need separate release branches to allow bug fixes for previous versions. This is important to support library users who can't upgrade to the latest version of OpenDP yet.

All release tasks should be automated and drivable from GitHub. Creating a release should not require setup of a local environment, and should not have any dependencies on non-standard tools. This is important to allow for delegation of tasks and continuity of the project.


Task Tracking
=============

* Use GitHub Issues to track all tasks. This is helpful to know who's working on what.
* Use the `OpenDP Development GitHub Project <https://github.com/orgs/opendp/projects/1?card_filter_query=label%3A%22opendp+core%22>`_ to organize work and prioritize tasks for development.
* Manage all changes using GitHub Pull Requests.

Code Hygiene
============

* Follow the Rust guidelines for coding style. Code should be formatted using the default settings of rustfmt. (TODO: Automatic marking of style issues on PRs -- https://github.com/opendp/opendp/issues/256)
* Write API docs for all public and significant private APIs. We use inline comments to explain complicated code.
* Make sure ``main`` is always in a good state: Code compiles, tests pass. If ``main`` is ever broken, it should be the team's top priority to fix it.
* Use GitHub Actions to check PRs automatically, and don't allow merges if checks fail.

Branching Strategy
==================

* Use a single, long-lived branch named ``main`` for core project history.
* Maintain a linear history on ``main``, meaning every commit is based only on the previous commit. Don't do merge commits onto ``main``.
* Do development on short-lived feature branches, derived from ``main``. Feature branches have the naming scheme ``<nnn>-<short-desc>``, where ``<nnn>`` is the number of the GitHub issue tracking this task, and ``<short-desc>`` is a short description of the change. For instance, ``123-new-measurement``
* Manage all changes using GitHub PRs from feature branches onto main. Check for test success and do code reviews before approving PRs.
* To maintain linear history, require PRs to be up to date with main. This means that developers may need to rebase feature branches periodically.
* Try to keep PRs relatively small and self contained. This simplifies code reviews, and reduces the likelihood of rebasing hassles.
* Generally, squash feature branches when merging PRs, so that there's a 1-to-1 correspondence between issues/PRs and commits.
* To enforce this strategy, use the following branch protections on main:

  * Require pull request reviews before merging
  * Dismiss stale pull request approvals when new commits are pushed
  * Require status checks to pass before merging
  * Require branches to be up to date before merging
  * Require linear history

* Because this is the real world, allow for exceptions to these rules in case of excessive misery!

Release Process
===============

Overview
--------

* For every release, designate a Release Manager. This person is charged with performing the key tasks of the release process. Responsibility for this should be rotated to avoid burnout.
* Use semantic versioning to identify all releases:

  * Golden master (GM) releases have a semantic version of the form ``<MAJ>.<MIN>.<PAT>``. For example, ``1.2.0``.
  * Release candidate (RC) releases have a semantic version of the form ``<MAJ>.<MIN>.<PAT>-rc.<NUM>``, where <NUM> starts at 1. For example, ``1.2.0-rc.1``.
  * The versions of the Rust crates and the Python package (and any other language bindings) are always kept in sync, even if there are no changes in one or the other. For example, version ``1.2.0`` will comprise Rust crates ``opendp 1.2.0`` and ``opendp-ffi 1.2.0``, and Python package ``opendp 1.2.0``.

* For major and minor releases, create a new release branch. Release branches have the naming scheme ``release/<MAJ>.<MIN>.x`` (where ``x`` is literal "``x``"). For example, version ``1.2.0`` → branch ``release/1.2.x``. Release branches remain alive as long as that minor version is supported. (Note: Release branch names *don't* contain the ``-rc.<NUM>`` suffix, even when doing an RC release. All RC releases and the GM release use the same branch.)
* For patch releases, don’t create a new release branch. Use the existing branch for the corresponding major or minor version. For example, version = ``1.2.1`` → branch ``release/1.2.x``.
* For all releases, create a tag. Tags have the naming scheme ``v<MAJ>.<MIN>.<PAT>[-rc.<NUM>]``. For example, for version = ``1.2.0``, tag = ``v1.2.0``. (Note: Tag names *do* contain the ``-rc.<NUM>`` suffix, when doing an RC release.)
* Use RC releases to validate the system end-to-end before creating the GM release. There should be at least one successful RC release before creating the GM release.
* Use a GitHub Release to initiate each OpenDP release. This will run the GitHub Workflows that handle the build and publish process (see below).

Playbook
--------

#. Identify names:

   ==============  ===============================  =================  ==============================
   Item            Format                           Example            Notes
   ==============  ===============================  =================  ==============================
   base version    ``<MAJ>.<MIN>.<PAT>``            ``1.2.0``          base for RC and GM versions
   release branch  ``release/<MAJ>.<MIN>.x``        ``release/1.2.x``  branch used for all iterations
   RC version      ``<MAJ>.<MIN>.<PAT>-rc.<NUM>``   ``1.2.0-rc.1``     incremented for each RC
   RC tag          ``v<MAJ>.<MIN>.<PAT>-rc.<NUM>``  ``v1.2.0-rc.1``
   GM version      ``<MAJ>.<MIN>.<PAT>``            ``1.2.0``          same as base version
   GM tag          ``v<MAJ>.<MIN>.<PAT>``           ``v1.2.0``
   ==============  ===============================  =================  ==============================

#. Update ``CHANGELOG.md`` on ``main`` (based on `Keep a Changelog <https://keepachangelog.com/en/1.0.0/>`_) .
#. Create/update the release branch:

   * Major or minor release ONLY: Create a *new* release branch, based on the desired point in ``main``.
   * Patch release ONLY: Use the *existing* branch from the previous major or minor release, and cherry-pick changes from ``main`` into the release branch.

#. Set the RC number to 1.
#. Specify the version for this iteration: ``<MAJ>.<MIN>.<PAT>[-rc.<NUM>]``
#. Update the version field(s) in the following files:

   * ``VERSION``
   * ``rust/opendp/Cargo.toml``
   * ``rust/opendp-ffi/Cargo.toml`` (two entries!!!)
   * ``python/setup.cfg``
   * ``docs/source/conf.py``

#. Commit the version number changes to the release branch.
#. Create a GitHub Release with the following parameters:

   :Tag version: ``v<MAJ>.<MIN>.<PAT>[-rc.<NUM>]``
   :Target: ``release/<MAJ>.<MIN>.<PAT>[-rc.<NUM>]``
   :Release title: ``OpenDP <MAJ>.<MIN>.<PAT>[-rc.<NUM>]``
   :Describe this release: (Changelog)[https://github.com/opendp/opendp/blob/main/CHANGELOG.md#<MAJ><MIN><PAT>---<ISO-8601-DATE>]
   :This is a pre-release: <CHECKED IF RC>
   :Create a discussion...: <UNCHECKED>

#. Build and publish process is triggered by the creation of the GitHub Release.
#. If this is a GM release, you're done!
#. If this is an RC release, download and sanity check the Rust crates and Python package. (TODO: Release validation scripts -- https://github.com/opendp/opendp/issues/251)
#. If fixes are necessary, do development on regular feature branches and merge them to ``main``, then cherry pick the fixes into the release branch.
#. Increment the RC number
#. Return to Step 4.


Release Workflows
-----------------

These are the GitHub workflows that support the release process.

sync-branches.yml
^^^^^^^^^^^^^^^^^

* Keeps the tracking branches ``latest`` and ``stable`` in sync with their targets. This is used when generating docs, so that we have a consistent path to each category.
* Triggered on every push to ``main``, or when release is published.
* Whenever there's a push to ``main``, it advances ``latest`` to the same ref.
* Whenever a release is created, it advances ``stable`` to the release tag.

release.yml
^^^^^^^^^^^

* Triggered whenever a GH Release is created.
* Rust library is compiled, creating shared libraries for Linux, macOS, Windows.
* Python package is created.
* Rust crates are uploaded to crates.io.
* Python packages are uploaded to PyPI.

docs.yml
^^^^^^^^

* Generates and publishes the docs to https://docs.opendp.org
* Triggered whenever ``sync-branches.html`` completes (i.e., whenever ``latest`` or ``stable`` have changed).
* Runs ``make versions``

  * Generates Python API docs
  * Generates Sphinx docs

* Pushes HTML to ``gh-pages`` branch, which is linked to https://docs.opendp.org
