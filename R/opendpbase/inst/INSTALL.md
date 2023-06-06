
### Install from Source

These steps are only useful when you want a full (fixed) package installation from source.
If you are doing library development, it is better to use the `(Re)load Package` instructions above.

To prepare the package for a full package installation, copy the rust sources into `src/rust`:

```shell
tools/r_stage.sh
```

To submit to CRAN, you must ensure that the checks pass:
```R
devtools::check()
```

Now that the package is staged, you can install it locally:
```R
devtools::install()
```

To restore to a developer setup (as described in the Development Environment docs) run:
```shell
tools/r_stage.sh -c
```

## Submit to CRAN

First, make sure that the checks pass (run from `R/opendpbase`):
```R
devtools::check()
```

To run the same check manually, use:
```bash
R CMD build .
R CMD check opendpbase_*.tar.xz --as-cran
```
It is important R CMD check is run on the `.tar.xz`, not on `.`, 
because `check` depends on some of the changes `build` makes within the `.tar.xz`.

## Testing on CI

Rust compilation is skipped when `libopendp.a` from `rust/target/debug/` is in `R/opendpbase/src/`.

```shell
cp rust/target/debug/libopendp.a R/opendpbase/src/
```

## Status 04.08.2023: Installing and making the package work
On linux stations, you might need to install rust using 

```shell
sudo apt-get update
sudo apt-get install cargo
```

otherwise R might not find cargo.

From the root directory of the openDP project, run the following command
```shell
bash tools/r_stage.sh && (cd R/opendpbase/ && Rscript -e 'devtools::install()')
cd R/opendpbase/
R
```

insert the following command in the opened R console:

```R
devtools::document()
```

The package should be installed and working (without pipe).


## Resources

* official resources:
    * https://cran.r-project.org/doc/manuals/r-devel/R-exts.html
    * https://cran.r-project.org/web/packages/using_rust.html
* uses yutannihilation's string2path as reference for Rust packaging:
    * https://github.com/yutannihilation/string2path
* blog series from yutannihilation (chronological):
    * https://yutani.rbind.io/post/2021-08-01-unofficial-introduction-to-extendr-appendix-i-setup-github-actions-ci-and-more/
    * https://yutani.rbind.io/post/2021-09-21-writing-a-configure-script-for-an-r-package-using-rust/
    * https://yutani.rbind.io/post/a-quick-note-about-how-to-bundle-rust/
* David B. Dahl: https://arxiv.org/pdf/2108.07179.pdf
* C implementation based on R Internals docs: 
    * https://cran.r-project.org/doc/manuals/r-devel/R-ints.html
* devtools <-> RStudio
    * https://stackoverflow.com/questions/44184068/devtools-equivalent-of-rstudio-build-panel-buttons
