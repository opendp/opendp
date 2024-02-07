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

def is_rst(line):
    """heuristic to determine where RST format begins"""
    return line.startswith(":") or line.startswith(".. ")

def docstring(app, what, name, obj, options, lines):
    path = name.split(".")

    if len(path) > 1 and path[1] in markdown_modules:
        # split docstring into description and params
        param_index = next((i for i, line in enumerate(lines) if is_rst(line)), len(lines))
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
   '**': ['sidebar-nav-bs.html', 'versioning.html'],
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

# insert this header on nbsphinx pages to link to binder and github:
# we have to resolve the link ref here, at runtime, because sphinx-multiversion mediates the reading of this config
nbsphinx_prolog = fr"""
{{% set docname = 'docs/source/' + env.doc2path(env.docname, base=None) %}}
{{% if env.config.release.endswith('-dev') %}}
    {{% set frag = 'main' %}}
{{% elif '-' in env.config.release %}}
    {{% set frag = env.config.release.split('-', 1)[1].split('.', 1)[0] %}}
{{% else %}}
    {{% set frag = 'v' ~ env.config.version %}}
{{% endif %}}
.. raw:: html

    <div class="admonition note">
      This page was generated from
      <a class="reference external" href="https://github.com/opendp/opendp/tree/{{{{ frag|e }}}}/{{{{ docname|e }}}}" target="_blank">{{{{ docname|e }}}}</a>.
      Interactive online version:
      <span style="white-space: nowrap;"><a href="https://mybinder.org/v2/gh/opendp/opendp/{{{{ frag|e }}}}?filepath={{{{ docname|e }}}}" target="_blank"><img alt="Binder badge" src="https://mybinder.org/badge_logo.svg" style="vertical-align:text-bottom"></a>.</span>
    </div>
"""
