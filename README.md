# OpenDP
[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![ci tests](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml/badge.svg)

[OpenDP](https://opendp.org) is a community effort to build trustworthy, open-source software tools for statistical analysis of sensitive private data. This repository contains OpenDP Core, a library of differential privacy algorithms powering OpenDP.

## *WARNING*
Please note that this library is still under development and not ready for production use. An important element of the OpenDP project is a formal vetting process that all library components must undergo, to verify their privacy characteristics. The code in this repository hasn't yet undergone that vetting. In addition to privacy concerns, it also likely has programming bugs.

Feel free to explore the code, but *please do not yet rely on it for any privacy-sensitive applications*.

## Overview
OpenDP Core is a library of differential privacy algorithms for performing statistical analysis of private data. It consists of two conceptual layers:

* A theoretical framework for expressing privacy-aware operations. This framework is described in the paper,
[A Programming Framework for OpenDP](https://projects.iq.harvard.edu/files/opendp/files/opendp_programming_framework_11may2020_1_01.pdf).

* A set of algorithmic components implemented within the conceptual framework. These components can be used "out of the box" to build applications for handling private data.

## Implementation
OpenDP Core is packaged as a library that can be incorporated into applications. Public APIs to the library are provided in Python and Rust, with more language bindings possible based on interest. The underlying logic of the library is implemented in native Rust.

## Status
OpenDP Core is early in its initial development. We're building out the main concepts of the OpenDP Programming Framework, and providing implementations of several common privacy mechanisms and summary statistics. Our focus is on correctness and developer usability.

The current code is functional, but not ready for general usage. The APIs are unstable and subject to extensive change. Aspects of the library are likely difficult to understand without examining the sources. Documentation and sample code is forthcoming.

## Communication
- You are very welcome to join us on [GitHub Discussions](https://github.com/opendp/opendp/discussions)!
- Please use [GitHub Issues](https://github.com/opendp/opendp/issues) for bug reports and feature requests.
- For other requests, including security issues, please contact us at [info@opendp.org](mailto:info@opendp.org).


## Install
- [From PyPI](https://pypi.org/project/opendp/)
- [From Crates.io](https://crates.io/crates/opendp)
- [From Source](https://docs.opendp.org/en/latest/resources/dev-guide/general-logistics/dev-environment.html)

## Documentation
- [Overview](https://docs.opendp.org)
- [Python API](https://docs.opendp.org/en/latest/api/python/index.html)
- [Rust API](https://docs.rs/opendp/)
    