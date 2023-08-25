
### Install from Source

These steps are only useful when you want a full (fixed) package installation from source.
If you are doing library development, it is better to use the `(Re)load Package` instructions above.

To do a full package installation from local sources:

```shell
bash tools/r_stage.sh && Rscript -e 'devtools::install("R/opendp")'
```

To restore to a developer setup (as described in the Development Environment docs) run:
```shell
tools/r_stage.sh -c
```

## Submit to CRAN

First, make sure that the checks pass:
```R
devtools::check("R/opendp")
```

To run the same check manually, use:
```bash
R CMD build R/opendp
R CMD check opendp_*.tar.gz --as-cran
```
It is important R CMD check is run on the `.tar.gz`, not on `R/opendp`, 
because `check` depends on some of the changes `build` makes within the `.tar.gz`.


## Troubleshooting
If R cannot find cargo, you might need to install Rust using:

```shell
sudo apt-get update
sudo apt-get install cargo
```



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
