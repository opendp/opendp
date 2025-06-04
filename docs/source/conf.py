# -*- coding: utf-8 -*-

import sys
import os
from datetime import datetime
import semver
import pypandoc
from sphinx.ext import autodoc

# docs should be built without needing import the library binary for the specified version
os.environ["OPENDP_HEADLESS"] = "true"

# We're inside source when this runs.
# Docs would be the same for all versions. Fix from: https://github.com/sphinx-contrib/multiversion/issues/42
rootdir = os.path.join(os.getenv("SPHINX_MULTIVERSION_SOURCEDIR", default=os.getcwd()), "..", "..", "python", "src")
sys.path.insert(0, rootdir)

# With sphinx-multiversion, the same configuration is used to build all versions of the documentation,
# so we need some extensions even though they are not currently used in main,
# and we should also be cautious about adding new extensions.
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
    'sphinx_design',
]

def docstring(app, what, name, obj, options, lines):
    flag = ".. end-markdown"
    
    for i, line in enumerate(lines):
        if line == flag:
            orig_md_lines, orig_rst_lines = lines[:i], lines[i:]
            new_rst = pypandoc.convert_text('\n'.join(orig_md_lines), 'rst', format='md')

            lines.clear()
            lines += new_rst.splitlines()
            lines += [""]
            lines += orig_rst_lines
            break
        

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

with open("../../VERSION") as f:
    semver_version = semver.Version.parse(f.read().strip())
# The version info for the project you're documenting, acts as replacement for
# |version| and |release|, also used in various other places throughout the
# built documents.
#
# The short X.Y version.
version = str(semver_version.replace(prerelease=None, build=None))
# The full version, including alpha/beta/rc tags.
release = str(semver_version)

nitpicky = True
nitpick_ignore = [
    # (no comment = single occurrence)
    
    # May be a problem with the stdlib?
    ('py:class', 'pathlib.Path'),

    # Maybe the quoted name is to prevent a circular reference?
    ('py:class', '"RuntimeType"'),  # 3 occurrences

    # Rather than a class, this is defined by setting a variable to a `Union[]`,
    # and we don't generate docs for variables, so there's nothing to point to.
    # Can we selectively turn on documentation for module variables?
    ('py:class', 'RuntimeTypeDescriptor'),  # 28 occurrences

    # For each of these, to provide a base class, the Python `Any*` class
    # is wrapped by ctypes.POINTER(), producing the `LP_*`,
    # which Sphinx can't resolve.
    ('py:class', 'opendp.mod.LP_AnyDomain'),
    ('py:class', 'opendp.mod.LP_AnyFunction'),
    ('py:class', 'opendp.mod.LP_AnyMeasure'),
    ('py:class', 'opendp.mod.LP_AnyMeasurement'),
    ('py:class', 'opendp.mod.LP_AnyMetric'),
    ('py:class', 'opendp.mod.LP_AnyTransformation'),

    # I think the problem is that Sphinx is making parameter list documentation,
    # and it doesn't understand that `M` and `T` are type parameters, not actual types.
    ('py:class', 'opendp.mod.M'),
    ('py:class', 'opendp.mod.T'),  # 17 occurrences

    # In a given version of Python, only one will apply,
    # but we need them both for compatibility.
    ('py:class', 'types.GenericAlias'),  # 56 occurrences
    ('py:obj', 'typing._GenericAlias'),  # 56 occurrences

    # nitpicky was first enabled in version 0.9
    # older versions contain these invalid references
    ('py:exc', 'UnknownTypeError'),
    ('py:func', 'opendp.mod.make_chain_mt'),
    ('py:func', 'opendp.mod.make_chain_tt'),
    ('py:func', 'opendp.trans.make_base_geometric'),
    ('py:func', 'opendp.trans.make_base_laplace'),
    ('py:func', 'opendp.trans.make_base_gaussian'),
    ('py:func', 'opendp.trans.make_base_stability'),
    ('py:func', 'opendp.meas.make_count_by'),
    ('py:func', 'make_count_by_categories'),
    ('py:func', 'opendp.meas.base_discrete_laplace'),
    ('py:func', 'opendp.measurements.make_count_by'),
    ('py:func', 'opendp.combinators.make_chain_tm'),
    ('py:func', 'opendp.measurements.make_count_by')
]

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
exclude_patterns = ['**/code/*.rst']

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
# Full list of options at https://pydata-sphinx-theme.readthedocs.io/en/stable/user_guide/layout.html#references
html_theme_options = {
    "github_url": "https://github.com/opendp",
    "article_header_end": ["questions-feedback", "old-version-warning"]
}

html_theme = 'pydata_sphinx_theme'
html_css_files = [
    'css/custom.css',
]

# See https://pydata-sphinx-theme.readthedocs.io/en/v0.6.3/user_guide/configuring.html#configure-the-sidebar
# Note: Overridden in the Makefile for local builds. Be sure to update both places.
html_sidebars = {
   '**': ['sidebar-nav-bs.html', 'versioning.html'],
}
html_context = {
    # Expected sphinx-multiversion to set "latest_version", but it was None, so set it manually.
    'latest_version_name': f'v{version}',
    # Uncomment this to see the old-version-warning:
    # 'current_version': 'some-old-version'
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

# Command that sphinx-multiversion runs for each version. Requires patch from https://github.com/sphinx-contrib/multiversion/pull/62
# We use this to generate the templates for the Python API docs.
# Because we need values to be calculated for each version, we can't use Python variables, so we have the shell expand them.
version_cmd = 'VERSION=`cat ../VERSION`'
# If sphinx-apidoc options change, also update Makefile!
sphinx_apidoc_cmd = 'sphinx-apidoc -f -F -e -d 3 -H "OpenDP" -A "The OpenDP Project" -V $VERSION -o source/api/python ../python/src/opendp --templatedir source/_templates'
smv_prebuild_command = '&&'.join([version_cmd, sphinx_apidoc_cmd])

# This is the file name suffix for HTML files (e.g. ".xhtml").
# html_file_suffix = None
htmlhelp_basename = 'OpenDPdoc'

html_logo = "_static/images/opendp-logo.png"

rst_prolog = """
.. |toctitle| replace:: Contents:
"""

class CustomClassDocumenter(autodoc.ClassDocumenter):
    '''
    Removes unneeded note from base classes.
    From https://stackoverflow.com/a/75041544/10727889
    '''
    def add_line(self, line: str, source: str, *lineno: int) -> None:
        if line == "   Bases: :py:class:`object`":
            return
        super().add_line(line, source, *lineno)

autodoc.ClassDocumenter = CustomClassDocumenter