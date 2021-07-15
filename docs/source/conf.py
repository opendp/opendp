# -*- coding: utf-8 -*-

import sys
import os
from datetime import datetime
sys.path.insert(0, os.path.abspath('../../'))

rootdir = os.path.join(os.getenv("SPHINX_MULTIVERSION_SOURCEDIR", default=os.getcwd()), "..")
sys.path.insert(0, rootdir)
sys.path.append(os.path.abspath('../../python/src'))
print("*****************************************")
[print(p) for p in sys.path]
print("*****************************************")

import opendp.smartnoise
import opendp.smartnoise.synthesizers
print(opendp.smartnoise.__spec__)
print("*****************************************")

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.doctest',
    'sphinx.ext.extlinks',
    'sphinx.ext.intersphinx',
    'sphinx.ext.ifconfig',
    'sphinx.ext.viewcode',
    'sphinx.ext.graphviz',
    'sphinx.ext.todo',
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
version = '0.0.1'
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

# Whitelist pattern for branches (set to None to ignore all branches)
# TODO: We would rather have "latest" than "main".
#smv_branch_whitelist = r'^latest$'
#smv_branch_whitelist = r'^main$'

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# This is the file name suffix for HTML files (e.g. ".xhtml").
#html_file_suffix = None
htmlhelp_basename = 'OpenDPdoc'

html_logo = "_static/images/opendp-logo.png"

rst_prolog = """
.. |toctitle| replace:: Contents:
.. |anotherSub| replace:: Yes, there can be multiple.
"""
