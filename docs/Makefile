# Makefile for Sphinx documentation
#

# You can set these variables from the command line.
CARGO         = cargo
SPHINXAPIDOC  = sphinx-apidoc
SPHINXOPTS    = -W
SPHINXBUILD   = sphinx-build
SPHINXMULTI   = sphinx-multiversion
PAPER         =
BUILDDIR      = build
VERSION       = $(shell cat ../VERSION)

# User-friendly check for sphinx-build
ifeq ($(shell which $(SPHINXBUILD) >/dev/null 2>&1; echo $$?), 1)
$(error The '$(SPHINXBUILD)' command was not found. Make sure you have Sphinx installed, then set the SPHINXBUILD environment variable to point to the full path of the '$(SPHINXBUILD)' executable. Alternatively you can add the directory with the executable to your PATH. If you don't have Sphinx installed, grab it from http://sphinx-doc.org/)
endif

.PHONY: help clean html dirhtml singlehtml pickle json htmlhelp qthelp devhelp epub latex latexpdf text man changes linkcheck doctest gettext

help:
	@echo "Please use \`make <target>' where <target> is one of"
	@echo "  html       to make HTML files for current workspace (FOR TESTING, NOT USED IN RELEASE BUILDS)"
	@echo "  versions   to make HTML files for all committed versions"

clean:
	rm -rf $(BUILDDIR)/* source/api/python source/api/rust

html:
	$(SPHINXAPIDOC) -f -F -e -H "OpenDP" -A "The OpenDP Project" -V $(VERSION) -o source/api/python ../python/src/opendp --templatedir source/_templates
# TODO: Figure out how to link locally generated Rust docs into sphinx structure.
# 	cd ../rust && $(CARGO) doc --no-deps --target-dir ../docs/source/api/rust
	$(SPHINXBUILD) $(SPHINXOPTS) -D version=$(VERSION) -D 'html_sidebars.**'=search-field.html,sidebar-nav-bs.html source $(BUILDDIR)/html
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."

doctest:
	cd ../rust && $(CARGO) test --doc
	$(SPHINXBUILD) $(SPHINXOPTS) -b doctest source $(BUILDDIR)/doctest
	@echo
	@echo "Doctest finished. The output is in $(BUILDDIR)/doctest/output.txt."

versions:
	# "/en" is the default, but someday we might have translations
	$(SPHINXMULTI) $(SPHINXOPTS) source $(BUILDDIR)/html/en
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."
	cp redirect.html $(BUILDDIR)/html/index.html
	@echo "Redirect page copied into $(BUILDDIR)/html."
