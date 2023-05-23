[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![CI](https://github.com/opendp/opendp-documentation/actions/workflows/main.yml/badge.svg)

# OpenDP Documentation

Note: The OpenDP documentation, [docs.opendp.org](https://docs.opendp.org), is currently under development.

## Building the Docs

The steps below assume the use of [Homebrew] on a Mac.

[Homebrew]: https://brew.sh

```shell
brew install pandoc
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
make html
open build/html/index.html
```

Make Sphinx html:
```shell
make html
```

Make html and run Python doctests:
```shell
make doctest-python
```

Make rust html:
```shell
make html-rustdoc
```
The output is located in `../rust/target/doc`.

## Simulating documentation sites locally
It is possible to fully simulate all documentation locally, 
with functioning links across documentation sites and proofs.

If the following environment variables are set, Sphinx and Rustdoc will build with links to locally-hosted docs:
```shell
export OPENDP_SPHINX_PORT=8020;
export OPENDP_RUSTDOC_PORT=8021;
```

Build all .tex files and copy them into the Sphinx documentation site:
```shell
make latex
```

Start two servers that host a local Sphinx docs site and local Rustdoc site:
```shell
make server
```

Putting these commands together, to update all docs sites and proofs, run:
```shell
make html html-rustdoc latex
```
Changes should automatically manifest without restarting the server.

## Deployment

Docs are deployed to http://docs.opendp.org using GitHub Actions.

Note that `make html` is replaced with `make versions` to build multiple versions (branches, tags) using the [sphinx-multiversion][] extension.
Be sure you have installed sphinx-multiversion from the fork in requirements.txt. 
Otherwise, you will get an error that includes: 

    /docs/source/api/index.rst:4:toctree contains reference to nonexisting document 'api/python/index'


[sphinx-multiversion]: https://holzhaus.github.io/sphinx-multiversion/

## Join the Discussion

You are very welcome to join us on [GitHub Discussions][]!

[GitHub Discussions]: https://github.com/opendp/opendp/discussions
