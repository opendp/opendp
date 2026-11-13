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

1. Make desired changes to the Markdown files in this directory.
2. Bump the version number in the file names, and internally on the last line of the file.
3. Generate new PDF files into the [`_static` directory](../source/_static) (requires `pandoc`):
    ```bash
    V='1.0.X' # Fill in correct number.
    pandoc opendp-cla-individual-$V.md -o ../source/_static/opendp-cla-individual-$V.pdf
    pandoc opendp-cla-company-$V.md -o ../source/_static/opendp-cla-company-$V.pdf
    ```
4. Commit these changes to `main`.
5. Update the URL references for the new version number in these files:
   * [Main CLA file](../source/contributing/cla.rst)
   * [Individual Workflow](https://github.com/opendp/clabot-config/blob/main/.github/workflows/sign-individual.yml)
   * [Company Workflow](https://github.com/opendp/clabot-config/blob/main/.github/workflows/sign-company.yml)
   * [`cla_tool.py`](https://github.com/opendp/clabot-config/blob/main/tools/cla_tool.py)
