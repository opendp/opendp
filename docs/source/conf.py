# -*- coding: utf-8 -*-

import sys
import os
from datetime import datetime
import semver

# docs should be built without needing import the library binary for the specified version
os.environ["OPENDP_HEADLESS"] = "true"

# We're inside source when this runs.
# Docs would be the same for all versions. Fix from: https://github.com/Holzhaus/sphinx-multiversion/issues/42
rootdir = os.path.join(os.getenv("SPHINX_MULTIVERSION_SOURCEDIR", default=os.getcwd()), "..", "..", "python", "src")
sys.path.insert(0, rootdir)

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.doctest',
    'sphinx.ext.extlinks',
    'sphinx.ext.graphviz',
    'sphinx.ext.ifconfig',
    'sphinx.ext.intersphinx',
    'sphinx.ext.viewcode',
    'sphinx.ext.todo',
    'sphinx-prompt',
    'sphinx_multiversion',
    'nbsphinx',
]

# convert markdown to rst when rendering with sphinx
markdown_modules = {
    "accuracy", 
    "combinators", 
    "core",
    "measurements", 
    "transformations",
    "domains",
    "metrics",
    "measures"
}

import pypandoc
import re
py_attr_re = re.compile(r"\:py\:\w+\:(``[^:`]+``)")

def docstring(app, what, name, obj, options, lines):
    path = name.split(".")

    if len(path) > 1 and path[1] in markdown_modules:
        # split docstring into description and params
        param_index = next((i for i, line in enumerate(lines) if line.startswith(":")), len(lines))
        description, params = lines[:param_index], lines[param_index:]
        
        # rust documentation is markdown, convert to rst
        rst = pypandoc.convert_text('\n'.join(description), 'rst', format='md')

        # allow sphinx notation to pass through
        params = "\n".join(line.replace("`", "``") for line in params)
        indexes = set()
        for match in py_attr_re.finditer(params):
            a, b = match.span(1)
            indexes |= {a, b - 1}
        params = "".join(l for i, l in enumerate(params) if i not in indexes)

        lines.clear()
        lines += rst.splitlines()
        lines += [""]
        lines += params.splitlines()

def setup(app):
    app.connect('autodoc-process-docstring', docstring)

# This prevents the RuntimeTypeDescriptors from expanding and making the signatures on API docs unreadable
autodoc_typehints = "description"

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# The suffix of source filenames.
source_suffix = '.rst'

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = u'OpenDP'
html_favicon = u'favicon.ico'
copyright = u'%d' % datetime.now().year

# The version info for the project you're documenting, acts as replacement for
# |version| and |release|, also used in various other places throughout the
# built documents.
#
# The short X.Y version.
with open("../VERSION") as f:
    version_str = f.read().strip()
version = semver.Version.parse(version_str)
# The full version, including alpha/beta/rc tags.
#release = ''

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
exclude_patterns = []

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# -- Options for HTML output ----------------------------------------------

# The name for this set of Sphinx documents.  If None, it defaults to
# "<project> v<release> Documentation".
html_title = 'OpenDP'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']

# If not '', a 'Last updated on:' timestamp is inserted at every page bottom,
# using the given strftime format.
html_last_updated_fmt = '%b %d, %Y'

# Custom sidebar templates, maps document names to template names.
html_theme_options = {
    "icon_links": [
        {
            "name": "GitHub Discussions",
            "url": "https://github.com/opendp/opendp/discussions",
            "icon": "far fa-comments",
        },
    ],
    "twitter_url": "https://twitter.com/opendp_org",
    "github_url": "https://github.com/opendp"
}

html_theme = 'pydata_sphinx_theme'
html_css_files = [
    'css/custom.css',
]

# See https://pydata-sphinx-theme.readthedocs.io/en/v0.6.3/user_guide/configuring.html#configure-the-sidebar
# Note: Overridden in the Makefile for local builds. Be sure to update both places.
html_sidebars = {
   '**': ['search-field.html', 'sidebar-nav-bs.html', 'versioning.html'],
}

# SPHINX-MULTIVERSION STUFF
# Whitelist pattern for tags (set to None to ignore all tags)
smv_tag_whitelist = r'^v.*$'
# # keep all released versions, as well as prereleases for the stable version. Doesn't work, because version != stable
# import re
# smv_tag_whitelist = rf'(^v\d+\.\d+\.\d+$)|(^v{re.escape(version.split("-")[0])}.+$)'

# Whitelist pattern for branches (set to None to ignore all branches)
smv_branch_whitelist = r'(stable|beta|nightly)'

# Whitelist pattern for remotes (set to None to use local branches only)
smv_remote_whitelist = r'origin'

# Pattern for released versions
smv_released_pattern = r'^tags/v\d+\.\d+\.\d+$'

# Command that sphinx-multiversion runs for each version. Requires patch from https://github.com/Holzhaus/sphinx-multiversion/pull/62
# We use this to generate the templates for the Python API docs.
# Because we need values to be calculated for each version, we can't use Python variables, so we have the shell expand them.
version_cmd = 'VERSION=`cat ../VERSION`'
sphinx_apidoc_cmd = 'sphinx-apidoc -f -F -e -H "OpenDP" -A "The OpenDP Project" -V $VERSION -o source/api/python ../python/src/opendp --templatedir source/_templates'
rustdoc_cmd = '(cd ../rust && cargo rustdoc --no-deps --target-dir ../docs/source/api/rust -- --html-in-header katex.html)'
# Building the Rust docs locally takes forever, and is only necessary for latest branch (releases are automatically published to https://docs.rs).
# TODO: Figure out how to use locally generated Rust docs for latest branch only.
#smv_prebuild_command = '&&'.join([version_cmd, sphinx_apidoc_cmd, rustdoc_cmd])
smv_prebuild_command = '&&'.join([version_cmd, sphinx_apidoc_cmd])

# This is the file name suffix for HTML files (e.g. ".xhtml").
#html_file_suffix = None
htmlhelp_basename = 'OpenDPdoc'

html_logo = "_static/images/opendp-logo.png"

rst_prolog = """
.. |toctitle| replace:: Contents:
"""

if version.prerelease is None:
    ref = f"v{version}"
else:
    ref = version.prerelease.split(".", 1)[0]
    if ref not in ("beta", "nightly"):
        print(f"Unexpected prerelease tag {version.prerelease}", file=sys.stderr)

github_frag = f'/tree/{ref}'
binder_frag = f'/{ref}'

# insert this header on nbsphinx pages to link to binder and github:
nbsphinx_prolog = fr"""
{{% set docname = 'docs/source/' + env.doc2path(env.docname, base=None) %}}
.. raw:: html

    <div class="admonition note">
      This page was generated from
      <a class="reference external" href="https://github.com/opendp/opendp{github_frag}/{{{{ docname|e }}}}" target="_blank">{{{{ docname|e }}}}</a>.
      Interactive online version:
      <span style="white-space: nowrap;"><a href="https://mybinder.org/v2/gh/opendp/opendp{binder_frag}?filepath={{{{ docname|e }}}}" target="_blank"><img alt="Binder badge" src="https://mybinder.org/badge_logo.svg" style="vertical-align:text-bottom"></a>.</span>
    </div>
"""
