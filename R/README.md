Work-in-progress R bindings for OpenDP.

1. Install devtools:

    ```R   
    install.packages("devtools")
    ```

2. Install OpenDP R package:

    ```R
    # - If you've already built, and only made changes to the rust code, compile_dll will do nothing
    # - This removes existing dll's, so that compile_dll will run again
    pkgbuild::clean_dll()
    # - runs src/Makevars
    # - builds libopendp.a and opendp.h
    # - copies opendp.h into src/
    # - compiles wrapper.c, which includes the two .h files and statically links with libopendp.a
    # - outputs src/opendp.so, which contains the extern function `slice_as_object__wrapper`
    pkgbuild::compile_dll()
    # - uses roxygen to generate `R/man` pages from #' comments
    # - loads the library into the current env
    # - must be done after every compile_dll(), or else the package will be stale  
    devtools::document()
    ```

3. Call into the rust OpenDP library:

    ```R
    data <- c(1.2, 2.3)
    opendp:::slice_as_object(data)
    opendp:::slice_as_object(as.integer(data))
    ```

    This should emit:
    > REALSXP  
    > Data inside rust: [1.2, 2.3]  
    > Success or error: 0  
    > [1] 1.2 2.3  
    > INTSXP  
    > Data inside rust: [1, 2]  
    > Success or error: 0  
    > [1] 1 2  
    
    1. Each call to slice_as_object first prints the typename, if recognized.  
    1. It then extracts the data from the SEXP into an FfiSlice, and passes the FfiSlice into a function in the OpenDP library.  
    1. The OpenDP library rust code reads the FfiSlice, interprets the contents based on the typename, prints the interpreted data, 
        packages the data into an AnyObject (or error) and returns the AnyObject (or error).
    1. The execution returns to wrapper.c, where the resulting structure is checked for an error.
    1. wrapper.c finally returns the input SEXP unmodified, which gets printed out into the repl. 

The following command currently fails:

```R
devtools::install()
```

It fails because Makevars uses a relative path to the rust directory.
When the package is installed, the directory is first lifted away to ensure that the package builds in isolation.
There are a couple solutions:
1. Add a symlink from /R/src/rust to /rust  
   - Would have to be a relative link to work with git, which similarly breaks on install  
   - Would cause cross platform issues w/ windows  
1. Copy the /rust into /R/src  
   - Slow to copy and easily gets de-synchronized  
1. Only copy /rust into /R/src when deploying the package  
   - The typical dev loop is `devtools::load_all()`. 
   - Never install the dev package  
   - This option seems ideal so far
1. Something else? What is the best practice here?


### Major Components to implement
The python library is generally a pretty good model for how this can be implemented.

1. Low-level type conversions between the bindings language and Rust  
    `python/src/_convert.py` -> `R/src/wrapper.c` or `R/src/convert.c`  
    The bulk of this is wrapping and unwrapping SEXPR types
2. Codegen for R constructor functions  
    `rust/build/python` -> `rust/build/R`  
    Constructor functions in `meas.py`, `trans.py`, `data.py` and `comb.py`
    are all generated from bootstrap.json metadata.
    Similarly in R, `meas.R`, `trans.R`, etc. can be generated into `R/inst`  
    It's best to implement a number of these constructors by hand first.
3. Tools for manipulating types idiomatically in R  
   `python/src/typing.py` -> `R/src/typing.R`
4. Adjust smoke-test.yml CI to automatically run testthat R/tests
