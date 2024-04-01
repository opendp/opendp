# OpenDP CLA Source Files

These are the source files for the CLA forms.
(See the [`cla-config` repo](https://github.com/opendp/clabot-config) for details on how CLA verification works.)

## Overview

The files here are derived from the original [PDF CLA](cla_opendp_project_2021.pdf).
This was converted to Markdown format, for ease of editing, and then split into two variations,
one for individual contributors and one for company contributors.
These Markdown files are then used to generate new PDFs that go into the [`_static` directory](../source/_static),
which is what actually gets published to the docs website.

## Modifications

In the unlikely event that we need to modify the CLA, these are the steps to follow:

1. Bump the version number.
2. Edit the Markdown files here.
3. Generate new PDF files into the [`_static` directory](../source/_static) (requires `pandoc`):
    ```
    $ pandoc opendp-cla-individual-X.Y.Z.md -o ../source/_static/opendp-cla-individual-X.Y.Z.pdf
    $ pandoc opendp-cla-company-X.Y.Z.md -o ../source/_static/opendp-cla-pandoc-X.Y.Z.pdf
    ```
4. Update the URL references for the new version number in these files:
   * [Main CLA file](../source/contributing/cla.rst)
   * [Individual Workflow](https://github.com/opendp/clabot-config/blob/main/.github/workflows/sign-individual.yml)
   * [Company Workflow](https://github.com/opendp/clabot-config/blob/main/.github/workflows/sign-company.yml)
   * [`cla_tool.py`](https://github.com/opendp/clabot-config/blob/main/tools/cla_tool.py)
