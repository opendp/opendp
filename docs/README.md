[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![CI](https://github.com/opendp/opendp-documentation/actions/workflows/main.yml/badge.svg)

# OpenDP Documentation

Note: The OpenDP documentation, [docs.opendp.org](https://docs.opendp.org), is currently under development.

## Building the Docs

The steps below assume the use of [Homebrew] on a Mac.

[Homebrew]: https://brew.sh

```shell
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
make html
open build/html/index.html
```

To make html and run python doctests:

```shell
make doctest-python
```

## Deployment

Docs are deployed to http://docs.opendp.org using GitHub Actions.

Note that `make html` is replaced with `make versions` to build multiple versions (branches, tags) using the [sphinx-multiversion][] extension.

[sphinx-multiversion]: https://holzhaus.github.io/sphinx-multiversion/

## Join the Discussion

You are very welcome to join us on [GitHub Discussions][]!

[GitHub Discussions]: https://github.com/opendp/opendp/discussions
