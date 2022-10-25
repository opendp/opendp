.. _contribution-process:

Contribution Process
********************

There's no need to fear, because our contribution process is human!
We're aiming to keep this process approachable to people of all skill levels.
Without compromising the integrity of the library,
there is reasonable variation in how strictly we adhere to this process in accordance to the depth of your contribution.


Development Setup
-----------------
The first task to tackle is setting up the development environment.
We have detailed instructions in the :ref:`development-environment` section.

Checkout to a new feature branch when you are ready to work on your issue.
Your branch should have a name that includes the issue number followed by a short description such as ``123-typo-in-readme``.

At this point, make commits to your branch and push periodically as you see fit.
If new commits have been added to the main branch, please rebase to help maintain a clean history.

Implementation
--------------
The programming framework suggests `many types of contributions <https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf#section.10>`_,
but this guide focuses on constructor contributions (types 1-3) for brevity.
The process involved for other kinds of code contribution will share similarities.

The :ref:`code-structure` section contains information about the various components you may be involved with:
the proof, constructor and FFI.

* If you are adding a new constructor, all three components are relevant.
* If you are resolving an issue, your work may be isolated to specific component(s).
  Even if you don't need to make changes to another component,
  be intentional about checking that your changes are consistent with the other components.

Submit your pull request once you are ready for feedback and/or review.
`GitHub has thorough documentation for this process when you are working with a fork <https://docs.github.com/en/github/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request-from-a-fork>`_.
You can use draft mode for PRs to make the state of your PR less ambiguous.
When you are ready for the review process, make sure your PR is undrafted and indicate that you are ready to begin.

Review Process
--------------
Reviewers will be checking that the requirements mentioned in :ref:`code-structure` are met,
so please be mindful of these to avoid snags in the review process.

After making your pull request, please check back for feedback from the core developers.

On the OpenDP side, the review process for the proof goes as follows:

#. Reviewers are assigned.
#. Reviewer confirms the privacy claims of the proof.
#. Reviewer validates the pseudo-code against the proof.
#. Reviewer validates the Rust code against the pseudo-code.

Please be patient— this process may take several iterations as issues come up in the review process.

Merge
-----

Once the review process is successful, your PR will be merged into the `main` branch.
Your contribution and credit will be added to the release notes,
and your changes will appear on crates.io and PyPi on our next library release.
