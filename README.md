# OpenDP
[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![ci tests](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml/badge.svg)

OpenDP is a modular library of statistical algorithms that adhere to the definition of [differential privacy](https://en.wikipedia.org/wiki/Differential_privacy). It can be used to build applications of privacy-preserving computations, using a number of different models of privacy. OpenDP is implemented in Rust, with bindings for easy use from Python.

The architecture of OpenDP is based on a conceptual framework for expressing privacy-aware computations. This framework is described in the paper
[A Programming Framework for OpenDP](https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf).

OpenDP (the library) is part of the larger [OpenDP Project](https://opendp.org), a community effort to build trustworthy, open source software tools for analysis of private data.

## Status

OpenDP is under development, and we aim to [release new versions](https://github.com/opendp/opendp/releases) frequently. It's a work in progress, but we feel that OpenDP already can be used to build some useful applications. We also hope that it may be a vehicle for exploring new ideas in privacy. We welcome you to try it and look forward to feedback on the usability of the library! However, please be aware of the following limitations.

### WARNING

OpenDP is not yet recommended for production deployments. It has known privacy concerns, particularly around floating point handling and side channel attacks. We expect to address these issues, but until then, *please don't use OpenDP for any privacy-critical applications*.

More details can be found in the [Limitations section of the User Guide](https://docs.opendp.org/en/stable/user/limitations.html)

## Installation

The easiest way to install OpenDP is using `pip` (the [package installer for Python](https://pypi.org/project/pip/)):

    % pip install opendp

More information can be found in the [Getting Started section of the User Guide](https://docs.opendp.org/en/stable/user/getting-started.html)

## Documentation

The full documentation for OpenDP is located at https://docs.opendp.org. Here are some helpful entry points:

* [User Guide](https://docs.opendp.org/en/stable/user/index.html)
* [Python API Docs](https://docs.opendp.org/en/stable/api/python/index.html)
* [Developer Guide](https://docs.opendp.org/en/stable/developer/index.html)

## Getting Help

If you're having problems using OpenDP, or want to submit feedback, please reach out! Here are some ways to contact us:

* Ask questions on our [discussions forum](https://github.com/opendp/opendp/discussions)
* Open issues on our [issue tracker](https://github.com/opendp/opendp/issues)
* Send critical bugs to [security@opendp.org](mailto:security@opendp.org)
* Send general queries to [info@opendp.org](mailto:info@opendp.org)
* Reach us on Twitter at [@opendp_org](https://twitter.com/opendp_org)

## Contributing

OpenDP is a community effort, and we welcome your contributions to its development! If you'd like to participate, please see the [Contributing section of the Developer Guide](https://docs.opendp.org/en/stable/developer/contributing.html)
