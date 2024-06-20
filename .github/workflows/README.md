# OpenDP workflows

## Summary

OpenDP uses a number of Github workflows: Some are started automaticly
(pushes, PRs, and scheduled runs), and others are manual (via the github UI or API).
The [Maintainer Notes](https://docs.opendp.org/en/nightly/contributing/maintainer-notes.html)
provide a broader view of the the develpment and release process.

### Automatic

#### Every PR

- [`smoke-test.yml`](smoke-test.yml) runs all tests and static checks. [![smoke-test status](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml/badge.svg)](https://github.com/opendp/opendp/actions/workflows/smoke-test.yml?query=branch%3Amain)
- [`latex.yml`](latex.yml) confirms that the LaTEX documentation builds, if there are changes to source files.

#### Nightly

- [`nightly.yml`](nightly.yml) makes a nightly release. [Past runs](https://github.com/opendp/opendp/actions/workflows/nightly.yml). [![nightly status](https://github.com/opendp/opendp/actions/workflows/nightly.yml/badge.svg)](https://github.com/opendp/opendp/actions/workflows/nightly.yml?query=branch%3Amain)
- [`docs.rs`](https://docs.rs/crate/opendp/latest) also builds the Rust documentation, separate from the GitHub CI. ![docs.rs status](https://img.shields.io/docsrs/opendp?label=docs.rs)

#### Weekly

- [`weekly-doc-check.yml`](weekly-doc-check.yml) checks external links. [Past runs](https://github.com/opendp/opendp/actions/workflows/weekly-doc-check.yml). [![weekly-doc-check status](https://github.com/opendp/opendp/actions/workflows/weekly-doc-check.yml/badge.svg)](https://github.com/opendp/opendp/actions/workflows/weekly-doc-check.yml?query=branch%3Amain)

### Manual

```mermaid
graph TD
    subgraph build.yml
        credential-check --> bindings
        bindings --> python
        bindings --> python-aarch64
        bindings --> python-sdist
        bindings --> r
    end
    build.yml -.- build

    subgraph credential-check.yml
       cccc[credential-check]
    end
    %% Adds a lot of crossing lines to the graph!
    %% credential-check.yml -.- bcc[credential-check]
    %% credential-check.yml -.- dcc[credential-check]
    %% credential-check.yml -.- lrcc[credential-check]
    %% credential-check.yml -.- lcc[credential-check]
    %% credential-check.yml -.- precc[credential-check]
    %% credential-check.yml -.- pubcc[credential-check]

    subgraph docs.yml
        dcc[credential-check] --> docs
        dcc[credential-check] --> r-docs
        r-docs --> docs
    end
    docs.yml -.- rdocs[docs]

    subgraph latex-release.yml
        lrcc[credential-check] --> latex
    end
    latex-release.yml -.- rlatex[latex]

    subgraph prepare.yml
        precc[credential-check] --> prepare
    end
    prepare.yml -.- rpre[prepare]

    subgraph publish.yml
        pubcc[credential-check] --> publish-python --> publish-rust --> publish-github
    end
    publish.yml -.- publish

    subgraph release.yml
        rpre[prepare] --> build --> sanity-test-pre --> publish
        rpre[prepare] --> publish --> sanity-test-post --> rdocs[docs]
        rpre[prepare] --> rlatex[latex] --> rdocs[docs]
    end

    subgraph sanity-test.yml
        sanity-test
    end
    sanity-test.yml -.- sanity-test-pre
    sanity-test.yml -.- sanity-test-post
```

#### `release.yml`

- Triggered whenever a GH Release is created.
- Rust library is compiled, creating shared libraries for Linux, macOS, Windows.
- Python package is created.
- Rust crates are uploaded to crates.io.
- Python packages are uploaded to PyPI.

#### `docs.yml`

- Last step in `release.yml`
- Runs `make versions`
  - Generates Python API docs
  - Generates Sphinx docs
- Pushes HTML to gh-pages branch, which is linked to https://docs.opendp.org


## Making one-off releases

One-off releases can be made with the
[`release.yml` workflow](https://github.com/opendp/opendp/actions/workflows/release.yml)
on github, or with the `gh` command line tool. Parameters:

- **Target channel** controls how the release is tagged, and what semantic version is given to the release. There is a git branch with the same name for each channel.
- The **sync the Channel from upstream?** checkbox is for when you want to update the `nightly`, `beta` or `stable` branches.
- Update the **version counter** accordingly when you want to release multiple nightlies or betas in the same day.
- **Dry runs** get sent to test-pypi, and don't update the docs
- **Fake** is for developer convenience when debugging CI: it skips compilation and inserts dummy binaries instead
