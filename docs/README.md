# OpenDP Documentation

## Building the docs locally

### Prerequisites

The steps below assume the use of [Homebrew](https://brew.sh) on a Mac.

```shell
cd docs
brew install pandoc
brew install basictex

sudo tlmgr install physics
sudo tlmgr install tcolorbox
sudo tlmgr install environ
sudo tlmgr install catchfile

python3 -m venv .venv-docs
source .venv-docs/bin/activate
pip install -r requirements.txt
```

### Python docs

Once depencies are installed, to buid python docs:

```shell
make html
open build/html/index.html
```

This local build differs from the hosted documentation:
- `docs.yml` uses spinx-multiversion to build docs for all releases of OpenDP.
- `docs.yml` also builds R documentation; See the next section.

### R docs

We use pkgdown to build R documentation.
To build R docs locally, from the project root run

```shell
bash tools/r_stage.sh -d
```

### Rust docs

Instead of using GitHub CI and Pages to build and host the Rust documentation,
we rely on docs.rs for both. To build the Rust docs locally:

```shell
cd docs
make html-rustdoc
```

### LaTeX docs

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

## Building the docs with Github Actions

Docs are deployed to http://docs.opendp.org using GitHub Actions.

Note that `make html` is replaced with `make versions` to build multiple versions (branches, tags) using the [`sphinx-multiversion`](https://sphinx-contrib.github.io/multiversion) extension.
Be sure you have installed `sphinx-multiversion` from the fork in requirements.txt. 
Otherwise, you will get an error that includes: 

```
/docs/source/api/index.rst:4:toctree contains reference to nonexisting document 'api/python/index'
```

Version selection for `make versions` is computed before Sphinx starts by
`docs/tools/select_smv_versions.py`, then passed into `conf.py` via
`OPENDP_SMV_TAG_WHITELIST`. This keeps `conf.py` static when
`sphinx-multiversion` re-evaluates it inside per-ref temporary build directories.
