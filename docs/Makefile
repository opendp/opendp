# Makefile for Sphinx documentation
#

# You can set these variables from the command line.
SPHINXOPTS    = -W
SPHINXBUILD   = sphinx-build
SPHINXMULTI   = sphinx-multiversion
PAPER         =
BUILDDIR      = build

# User-friendly check for sphinx-build
ifeq ($(shell which $(SPHINXBUILD) >/dev/null 2>&1; echo $$?), 1)
$(error The '$(SPHINXBUILD)' command was not found. Make sure you have Sphinx installed, then set the SPHINXBUILD environment variable to point to the full path of the '$(SPHINXBUILD)' executable. Alternatively you can add the directory with the executable to your PATH. If you don't have Sphinx installed, grab it from http://sphinx-doc.org/)
endif

.PHONY: help clean html dirhtml singlehtml pickle json htmlhelp qthelp devhelp epub latex latexpdf text man changes linkcheck doctest gettext

help:
	@echo "Please use \`make <target>' where <target> is one of"
	@echo "  html       to make standalone HTML files"

clean:
	rm -rf $(BUILDDIR)/*

html:
	$(SPHINXBUILD) $(SPHINXOPTS) -D 'html_sidebars.**'=search-field.html,sidebar-nav-bs.html source $(BUILDDIR)/html
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."

versions:
	# "/en" is the default, but someday we might have translations
	$(SPHINXMULTI) $(SPHINXOPTS) source $(BUILDDIR)/html/en
	@echo
	@echo "Build finished. The HTML pages are in $(BUILDDIR)/html."
	cp redirect.html $(BUILDDIR)/html/index.html
	@echo "Redirect page copied into $(BUILDDIR)/html."
