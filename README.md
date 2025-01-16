<img src="https://docs.opendp.org/en/stable/_static/opendp-logo.png" width="200" alt="OpenDP logo">

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/license/MIT)

[![Python](https://img.shields.io/badge/Python-3.9%20%E2%80%93%203.12-blue)](https://docs.opendp.org/en/stable/api/python/index.html)
[![R](https://img.shields.io/badge/R-grey)](https://docs.opendp.org/en/stable/api/r/)
[![Rust](https://img.shields.io/badge/Rust-grey)](https://docs.rs/crate/opendp/latest)

[![main CI](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml?query=branch%3Amain)
[![nightly CI](https://github.com/opendp/opendp/actions/workflows/nightly.yml/badge.svg)](https://github.com/opendp/opendp/actions/workflows/nightly.yml?query=branch%3Amain)

The OpenDP Library is a modular collection of statistical algorithms that adhere to the definition of
[differential privacy](https://en.wikipedia.org/wiki/Differential_privacy).
It can be used to build applications of privacy-preserving computations, using a number of different models of privacy.
OpenDP is implemented in Rust, with bindings for easy use from Python and R.

The architecture of the OpenDP Library is based on a conceptual framework for expressing privacy-aware computations.
This framework is described in the paper [A Programming Framework for OpenDP](https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf).

The OpenDP Library is part of the larger [OpenDP Project](https://opendp.org), a community effort to build trustworthy,
open source software tools for analysis of private data.
(For simplicity in these docs, when we refer to “OpenDP,” we mean just the library, not the entire project.)

## Status

OpenDP is under development, and we expect to [release new versions](https://github.com/opendp/opendp/releases) frequently,
incorporating feedback and code contributions from the OpenDP Community.
It's a work in progress, but it can already be used to build some applications and to prototype contributions that will expand its functionality.
We welcome you to try it and look forward to feedback on the library! However, please be aware of the following limitations:

> OpenDP, like all real-world software, has both known and unknown issues.
> If you intend to use OpenDP for a privacy-critical application, you should evaluate the impact of these issues on your use case.
> 
> More details can be found in the [Limitations section of the User Guide](https://docs.opendp.org/en/stable/api/user-guide/limitations.html).


## Installation

Install OpenDP for Python with `pip` (the [package installer for Python](https://pypi.org/project/pip/)):

    $ pip install opendp

Install OpenDP for R from an R session:

    install.packages("opendp", repos = "https://opendp.r-universe.dev")

More information can be found in the [Getting Started section of the User Guide](https://docs.opendp.org/en/stable/getting-started/).

## Documentation

The full documentation for OpenDP is located at https://docs.opendp.org. Here are some helpful entry points:

* [User Guide](https://docs.opendp.org/en/stable/api/user-guide/index.html)
* [Python API Docs](https://docs.opendp.org/en/stable/api/python/index.html)
* [Contributor Guide](https://docs.opendp.org/en/stable/contributing/index.html)

## Getting Help

If you're having problems using OpenDP, or want to submit feedback, please reach out! Here are some ways to contact us:

<!--
    All of these lists should be in sync:
    - README.md
    - docs/source/contributing/contact.rst
    - docs/source/_templates/questions-feedback.html
    - .github/ISSUE_TEMPLATE/config.yml

    (although office hours are only listed here.)
-->

* Report a bug or request a feature on [Github](https://github.com/opendp/opendp/issues).
* Send general queries to [info@opendp.org](mailto:info@opendp.org), or email [security@opendp.org](mailto:security@opendp.org) if it is related to security.
* Join the conversation on [Slack](https://join.slack.com/t/opendp/shared_invite/zt-zw7o1k2s-dHg8NQE8WTfAGFnN_cwomA), or the [mailing list](https://groups.google.com/a/g.harvard.edu/g/opendp-community).
* Office hours are M/T/Th at 11am Eastern on [Zoom](https://harvard.zoom.us/j/98058847683).

## Contributing

OpenDP is a community effort, and we welcome your contributions to its development! 
If you'd like to participate, please contact us! We also have a [contribution process section in the Contributor Guide](https://docs.opendp.org/en/stable/contributing/contribution-process.html).
