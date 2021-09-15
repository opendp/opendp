# -*- coding: utf-8 -*-

import sys
import os
from datetime import datetime

# We're inside source when this runs.
sys.path.append(os.path.abspath('../../python/src'))
# print("*****************************************")
# [print(p) for p in sys.path]
# print("*****************************************")

print("*****************************************")

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
]

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# The suffix of source filenames.
source_suffix = '.rst'

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = u'OpenDP'
copyright = u'%d' % datetime.now().year

# The version info for the project you're documenting, acts as replacement for
# |version| and |release|, also used in various other places throughout the
# built documents.
#
# The short X.Y version.
version = '0.0.0-development'
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

# See https://pydata-sphinx-theme.readthedocs.io/en/v0.6.3/user_guide/configuring.html#configure-the-sidebar
# Note: Overridden in the Makefile for local builds. Be sure to update both places.
html_sidebars = {
   '**': ['search-field.html', 'sidebar-nav-bs.html', 'versioning.html'],
}

# SPHINX-MULTIVERSION STUFF
# Whitelist pattern for tags (set to None to ignore all tags)
smv_tag_whitelist = r'^v.*$'

# Whitelist pattern for branches (set to None to ignore all branches)
smv_branch_whitelist = r'(stable|latest)'

# Whitelist pattern for remotes (set to None to use local branches only)
smv_remote_whitelist = r'origin'

# Pattern for released versions
smv_released_pattern = r'^tags/v\d+\.\d+\.\d+$'

# Command that sphinx-multiversion runs for each version. Requires patch from https://github.com/Holzhaus/sphinx-multiversion/pull/62
# We use this to generate the templates for the Python API docs.
# Because we need values to be calculated for each version, we can't use Python variables, so we have the shell expand them.
version_cmd = 'VERSION=`cat ../VERSION`'
sphinx_apidoc_cmd = 'sphinx-apidoc -f -F -e -H "OpenDP" -A "The OpenDP Project" -V $VERSION -o source/api/python ../python/src/opendp --templatedir source/_templates'
rustdoc_cmd = '(cd ../rust && cargo doc --no-deps --target-dir ../docs/source/api/rust)'
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
.. |anotherSub| replace:: Yes, there can be multiple.
"""
